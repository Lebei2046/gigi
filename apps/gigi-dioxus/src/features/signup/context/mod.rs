use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum SignupType {
    Create,
    Import,
    None,
}

#[derive(Clone, PartialEq)]
pub struct SignupState {
    pub current_step: usize,
    pub steps: Vec<bool>,
    pub signup_type: SignupType,
    pub mnemonic: Vec<String>,
    pub password: String,
    pub address: String,
    pub peer_id: String,
    pub name: String,
    pub create_group: bool,
    pub group_name: String,
}

impl Default for SignupState {
    fn default() -> Self {
        Self {
            current_step: 0,
            steps: vec![false; 4],
            signup_type: SignupType::None,
            mnemonic: vec![String::new(); 12],
            password: String::new(),
            address: String::new(),
            peer_id: String::new(),
            name: String::new(),
            create_group: false,
            group_name: String::new(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum SignupAction {
    GoToNextStep,
    GoToPrevStep,
    SetMnemonic(Vec<String>),
    SetMnemonicWord(usize, String),
    SetPassword(String),
    SetName(String),
    SetAddress(String),
    SetPeerId(String),
    SetCreateGroup(bool),
    SetGroupName(String),
    SetStepChecked(usize, bool),
    InitSignup(SignupType),
}

pub fn signup_reducer(state: &SignupState, action: SignupAction) -> SignupState {
    match action {
        SignupAction::GoToNextStep => SignupState {
            current_step: state.current_step + 1,
            ..state.clone()
        },
        SignupAction::GoToPrevStep => SignupState {
            current_step: state.current_step.saturating_sub(1),
            signup_type: if state.current_step == 0 {
                SignupType::None
            } else {
                state.signup_type.clone()
            },
            ..state.clone()
        },
        SignupAction::SetMnemonic(mnemonic) => SignupState {
            mnemonic,
            ..state.clone()
        },
        SignupAction::SetMnemonicWord(index, word) => {
            let mut new_mnemonic = state.mnemonic.clone();
            if index < new_mnemonic.len() {
                new_mnemonic[index] = word;
            }
            SignupState {
                mnemonic: new_mnemonic,
                ..state.clone()
            }
        }
        SignupAction::SetPassword(password) => SignupState {
            password,
            ..state.clone()
        },
        SignupAction::SetName(name) => SignupState {
            name,
            ..state.clone()
        },
        SignupAction::SetCreateGroup(create_group) => SignupState {
            create_group,
            ..state.clone()
        },
        SignupAction::SetGroupName(group_name) => SignupState {
            group_name,
            ..state.clone()
        },
        SignupAction::InitSignup(signup_type) => SignupState {
            signup_type,
            steps: vec![false; 4],
            mnemonic: vec![String::new(); 12],
            password: String::new(),
            ..state.clone()
        },
        SignupAction::SetStepChecked(index, checked) => {
            let mut new_steps = state.steps.clone();
            if index < new_steps.len() {
                new_steps[index] = checked;
            }
            SignupState {
                steps: new_steps,
                ..state.clone()
            }
        }
        SignupAction::SetAddress(address) => SignupState {
            address,
            ..state.clone()
        },
        SignupAction::SetPeerId(peer_id) => SignupState {
            peer_id,
            ..state.clone()
        },
    }
}

#[derive(Clone)]
pub struct SignupContext {
    pub state: Signal<SignupState>,
    pub dispatch: Callback<SignupAction>,
    pub save_account_info: Callback<()>,
    pub save_group_info: Callback<()>,
}

pub fn use_signup_context() -> SignupContext {
    use_context::<SignupContext>()
}

#[component]
pub fn SignupProvider(children: Element) -> Element {
    let mut state = use_signal(|| SignupState::default());

    let dispatch = use_callback(move |action: SignupAction| {
        let mut state_write = state.write();
        *state_write = signup_reducer(&*state_write, action);
    });

    let save_account_info = use_callback(move |_| {
        let current_state = state.read();
        let mnemonic_str = current_state.mnemonic.join(" ");
        let password = current_state.password.clone();
        let name = current_state.name.clone();
        let create_group = current_state.create_group;
        let group_name = if create_group {
            Some(current_state.group_name.clone())
        } else {
            None
        };
        let dispatch_clone = dispatch.clone();

        spawn(async move {
            match crate::services::auth_service::AuthService::new().await {
                Ok(mut auth_service) => {
                    match auth_service
                        .create_account(&mnemonic_str, &password, &name, group_name.as_deref())
                        .await
                    {
                        Ok(account_info) => {
                            dispatch_clone.call(SignupAction::SetAddress(account_info.address));
                            dispatch_clone.call(SignupAction::SetPeerId(account_info.peer_id));
                        }
                        Err(err) => {
                            println!("Error creating account: {:?}", err);
                        }
                    }
                }
                Err(err) => {
                    println!("Error creating auth service: {:?}", err);
                }
            }
        });
    });

    let save_group_info = use_callback(move |_| {
        // Group is already created in saveAccountInfo if groupName was provided
        // This function is kept for compatibility but does nothing
        println!("Group info already saved during account creation");
    });

    let context = SignupContext {
        state,
        dispatch,
        save_account_info,
        save_group_info,
    };

    provide_context(context);
    rsx! {
        div { {children} }
    }
}
