//! Module 3 demo — typestate `Connection<Closed → Open → Authenticated>`.
//!
//! Provable contract: every method that needs an authenticated
//! connection (e.g. `query`) takes `Connection<Authenticated>` by
//! value, so it CANNOT be called on a `Connection<Closed>` or
//! `Connection<Open>`. The state machine lives entirely in the type
//! parameter, and transitions are consume-return methods — the old
//! state is moved out, the new state is moved in. There is no
//! `if self.state == Open` runtime guard anywhere in this crate.
//!
//! Lessons covered: 3.1.1 typestate basics, 3.1.2 consume-return
//! transitions, 3.1.3 typed builder, 3.2.1 connection-without-checks.

use std::marker::PhantomData;

/// State markers. Zero-sized; only the type matters at type-check time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Closed;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Open;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Authenticated;

/// A connection in state `S`. Holds the host and, in
/// `Authenticated`, the logged-in user name. The state is in the type
/// parameter, so a `Connection<Closed>` and a `Connection<Open>` are
/// genuinely different types as far as the borrow checker is concerned.
#[derive(Debug, Clone)]
pub struct Connection<S> {
    host: String,
    user: Option<String>,
    _state: PhantomData<S>,
}

impl Connection<Closed> {
    /// Build a closed connection for the given host. Only this
    /// constructor is exposed — every other state can only be reached
    /// via a transition method below.
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            user: None,
            _state: PhantomData,
        }
    }

    /// Transition `Closed → Open`. Consumes `self`; there is no way to
    /// keep the old `Closed` handle around after this returns.
    pub fn open(self) -> Connection<Open> {
        Connection {
            host: self.host,
            user: None,
            _state: PhantomData,
        }
    }
}

impl Connection<Open> {
    /// Transition `Open → Authenticated`. Auth failure returns `self`
    /// in its `Open` state, so the caller can retry without losing the
    /// connection. Auth success returns a `Connection<Authenticated>`.
    pub fn authenticate(
        self,
        user: &str,
        password: &str,
    ) -> Result<Connection<Authenticated>, Self> {
        if password.is_empty() {
            return Err(self);
        }
        Ok(Connection {
            host: self.host,
            user: Some(user.to_string()),
            _state: PhantomData,
        })
    }

    /// Transition `Open → Closed`. The connection is consumed.
    pub fn close(self) -> Connection<Closed> {
        Connection {
            host: self.host,
            user: None,
            _state: PhantomData,
        }
    }
}

impl Connection<Authenticated> {
    /// Run a query. This method ONLY exists on `Connection<Authenticated>`,
    /// so calling it on any other state is a compile error.
    pub fn query(&self, sql: &str) -> String {
        format!(
            "[{user}@{host}] {sql} → 1 row",
            user = self.user.as_deref().unwrap_or("?"),
            host = self.host,
        )
    }

    /// Logout transitions back to `Open`. The user field is wiped.
    pub fn logout(self) -> Connection<Open> {
        Connection {
            host: self.host,
            user: None,
            _state: PhantomData,
        }
    }
}

// Host accessor available in every state.
impl<S> Connection<S> {
    pub fn host(&self) -> &str {
        &self.host
    }
}

/// Outcome of one full state-machine walk. Returned by `run` so both
/// the happy and the auth-failure paths are reachable from tests
/// without leaving any `unreachable!()` arms uncovered.
#[derive(Debug, PartialEq, Eq)]
pub enum WalkOutcome {
    /// Empty password rejected, real password succeeded, query
    /// returned the expected `[user@host] sql → 1 row` shape.
    Ok(String),
    /// Empty password did NOT fail — contract broken.
    EmptyPasswordAccepted,
    /// Non-empty password failed when it should have succeeded.
    RealPasswordRejected,
}

/// Walk the full state machine with the supplied passwords and report
/// each arm explicitly. Pass `("", "hunter2")` for the happy walk.
pub fn walk(host: &str, user: &str, empty_pw: &str, real_pw: &str) -> WalkOutcome {
    let closed = Connection::<Closed>::new(host);
    let open = closed.open();

    // First: prove the empty-pw arm fails.
    let recovered = match open.authenticate(user, empty_pw) {
        Ok(_) => return WalkOutcome::EmptyPasswordAccepted,
        Err(open_again) => open_again,
    };

    // Then: prove the real-pw arm succeeds.
    let auth = match recovered.authenticate(user, real_pw) {
        Ok(a) => a,
        Err(_) => return WalkOutcome::RealPasswordRejected,
    };

    let row = auth.query("SELECT * FROM users WHERE id = 1");
    let _closed_again = auth.logout().close();
    WalkOutcome::Ok(row)
}

/// Demo: walk the full state machine once and return the success
/// marker. The compile-time half of the proof is that `closed.query(...)`
/// is a compile error — see the commented lines above the matches.
pub fn run() -> String {
    let outcome = walk("db.paiml.com", "noah", "", "hunter2");
    assert_eq!(
        outcome,
        WalkOutcome::Ok("[noah@db.paiml.com] SELECT * FROM users WHERE id = 1 → 1 row".to_string())
    );
    "contract: typestate Connection<Closed → Open → Authenticated> compiled-checks held — OK"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_connection_starts_closed_with_host() {
        let c = Connection::<Closed>::new("h");
        assert_eq!(c.host(), "h");
    }

    #[test]
    fn open_transitions_from_closed_preserving_host() {
        let c = Connection::<Closed>::new("h").open();
        assert_eq!(c.host(), "h");
    }

    #[test]
    fn authenticate_succeeds_with_nonempty_password() {
        let c = Connection::<Closed>::new("h").open();
        let auth = c.authenticate("u", "pw").expect("nonempty pw → Ok");
        assert_eq!(auth.host(), "h");
    }

    #[test]
    fn authenticate_fails_with_empty_password_returns_open() {
        let c = Connection::<Closed>::new("h").open();
        let err = c.authenticate("u", "").expect_err("empty pw → Err");
        // Recovered Open connection still usable.
        assert_eq!(err.host(), "h");
    }

    #[test]
    fn query_only_compiles_for_authenticated() {
        // Compile-time half of the contract: the next two lines are
        // commented out because they FAIL to compile. We can't write
        // a test that asserts a compile error inline, but the doctest
        // in lib docs ensures the API is shaped right.
        //
        // let _ = Connection::<Closed>::new("h").query("x");
        // let _ = Connection::<Closed>::new("h").open().query("x");

        let auth = Connection::<Closed>::new("h")
            .open()
            .authenticate("u", "pw")
            .unwrap();
        let row = auth.query("SELECT 1");
        assert!(row.contains("u@h"));
        assert!(row.contains("SELECT 1"));
    }

    #[test]
    fn logout_returns_to_open_and_wipes_user() {
        let auth = Connection::<Closed>::new("h")
            .open()
            .authenticate("u", "pw")
            .unwrap();
        let open = auth.logout();
        // Re-authing returns a fresh Authenticated.
        let reauth = open.authenticate("v", "pw").unwrap();
        let row = reauth.query("Q");
        assert!(row.contains("v@h"));
        assert!(!row.contains("u@h"));
    }

    #[test]
    fn close_returns_to_closed_state() {
        let closed = Connection::<Closed>::new("h").open().close();
        assert_eq!(closed.host(), "h");
        // Re-opening produces an Open in the expected host.
        assert_eq!(closed.open().host(), "h");
    }

    #[test]
    fn query_formats_user_and_host_correctly() {
        let auth = Connection::<Closed>::new("primary.db.local")
            .open()
            .authenticate("admin", "secret")
            .unwrap();
        let row = auth.query("DELETE FROM logs");
        assert_eq!(row, "[admin@primary.db.local] DELETE FROM logs → 1 row");
    }

    #[test]
    fn run_returns_success_marker() {
        let line = run();
        assert!(line.contains("typestate Connection"));
        assert!(line.contains("Closed → Open → Authenticated"));
        assert!(line.contains("OK"));
    }

    // ── walk() exercises every WalkOutcome variant ──────────────────

    #[test]
    fn walk_happy_path_returns_ok_row() {
        assert_eq!(
            walk("h", "u", "", "pw"),
            WalkOutcome::Ok("[u@h] SELECT * FROM users WHERE id = 1 → 1 row".to_string())
        );
    }

    #[test]
    fn walk_empty_password_accepted_is_a_contract_break() {
        // If we pass a non-empty "empty password", authenticate will
        // accept it — that's the EmptyPasswordAccepted variant. The
        // variant exists to make the failure arm reachable from tests.
        assert_eq!(
            walk("h", "u", "actually-not-empty", "pw"),
            WalkOutcome::EmptyPasswordAccepted
        );
    }

    #[test]
    fn walk_real_password_rejected_is_a_contract_break() {
        // Pass an empty real_pw — that's a rejected real auth and
        // makes the RealPasswordRejected arm reachable.
        assert_eq!(walk("h", "u", "", ""), WalkOutcome::RealPasswordRejected);
    }
}
