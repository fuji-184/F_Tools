use std::marker::PhantomData;

pub struct StateEventAction<S, E> {
    pub current: S,
    _marker: PhantomData<E>,
}

impl<S: Copy, E> StateEventAction<S, E> {
    pub fn new(initial: S) -> Self {
        Self {
            current: initial,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn dispatch<T>(&mut self, event: E, transition_fn: T)
    where
        T: Fn(S, E) -> S,
    {
        self.current = transition_fn(self.current, event);
    }
}

#[macro_export]
macro_rules! state_event_action {
    ($name:ident, $state_enum:ident, $event_enum:ident, { 
        $($curr:pat, $evt:pat => $next:expr $(, $action:block)? ),* 
    }) => {
        pub struct $name;

        impl $name {
            pub fn transition(state: $state_enum, event: $event_enum) -> $state_enum {
                match (state, event) {
                    $(
                        ($curr, $evt) => {
                            $( $action )?
                            $next
                        },
                    )*
                    (s, _) => s,
                }
            }
        }
    };
}

/* contoh penggunaan

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConnState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

pub enum ConnEvent {
    Connect,
    Success,
    Fail(u16),
    Terminate,
}

state_event_action!(ConnHandler, ConnState, ConnEvent, {
    ConnState::Disconnected, ConnEvent::Connect => ConnState::Connecting, {
        println!("Transitioning: Disconnected -> Connecting");
    },
    ConnState::Connecting, ConnEvent::Success => ConnState::Connected, {
        println!("Transitioning: Connecting -> Connected");
    },
    ConnState::Connecting, ConnEvent::Fail(code) => ConnState::Error, {
        println!("Error occurred with code: {}", code);
    },
    ConnState::Connected, ConnEvent::Terminate => ConnState::Disconnected, {
        println!("Transitioning: Connected -> Disconnected");
    },
    ConnState::Error, ConnEvent::Connect => ConnState::Connecting
});

fn main() {
    let mut fsm = StateEventAction::new(ConnState::Disconnected);

    fsm.dispatch(ConnEvent::Connect, ConnHandler::transition);
    fsm.dispatch(ConnEvent::Success, ConnHandler::transition);
    
    assert_eq!(fsm.current, ConnState::Connected);

    fsm.dispatch(ConnEvent::Terminate, ConnHandler::transition);
    assert_eq!(fsm.current, ConnState::Disconnected);
}

*/

ftest::test!(state_event_action_tests, {
    
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum ConnState {
        Disconnected,
        Connecting,
        Connected,
        Error,
    }
    
    pub enum ConnEvent {
        Connect,
        Success,
        Fail(u16),
        Terminate,
    }
    
    state_event_action!(ConnHandler, ConnState, ConnEvent, {
        ConnState::Disconnected, ConnEvent::Connect => ConnState::Connecting, {
            let _ = "Transitioning: Disconnected -> Connecting";
        },
        ConnState::Connecting, ConnEvent::Success => ConnState::Connected, {
            let _ = "Transitioning: Connecting -> Connected";
        },
        ConnState::Connecting, ConnEvent::Fail(_code) => ConnState::Error, {
            let _ = "Error occurred";
        },
        ConnState::Connected, ConnEvent::Terminate => ConnState::Disconnected, {
            let _ = "Transitioning: Connected -> Disconnected";
        },
        ConnState::Error, ConnEvent::Connect => ConnState::Connecting
    });
    

    test_state_initial_and_dispatch_success {
        let mut fsm = StateEventAction::new(ConnState::Disconnected);
        assert_eq!(fsm.current, ConnState::Disconnected);

        fsm.dispatch(ConnEvent::Connect, ConnHandler::transition);
        assert_eq!(fsm.current, ConnState::Connecting);

        fsm.dispatch(ConnEvent::Success, ConnHandler::transition);
        assert_eq!(fsm.current, ConnState::Connected);
    }

    test_state_dispatch_fail_to_error {
        let mut fsm = StateEventAction::new(ConnState::Connecting);

        fsm.dispatch(ConnEvent::Fail(500), ConnHandler::transition);
        assert_eq!(fsm.current, ConnState::Error);
    }

    test_state_invalid_transition_remains_unchanged {
        let mut fsm = StateEventAction::new(ConnState::Disconnected);

        fsm.dispatch(ConnEvent::Success, ConnHandler::transition);
        assert_eq!(fsm.current, ConnState::Disconnected);
    }

    test_state_terminate_connection {
        let mut fsm = StateEventAction::new(ConnState::Connected);

        fsm.dispatch(ConnEvent::Terminate, ConnHandler::transition);
        assert_eq!(fsm.current, ConnState::Disconnected);
    }
});