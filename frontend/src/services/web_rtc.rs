use wasm_bindgen::{JsCast, JsValue};

use std::cell::RefCell;
use std::rc::Rc;
use std::str;

use base64::{self, engine::general_purpose, Engine};
use js_sys::{Array, Object, Reflect, JSON};
use serde::{Deserialize, Serialize};
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    console, RtcConfiguration, RtcDataChannel, RtcDataChannelEvent, RtcDataChannelInit,
    RtcDataChannelState, RtcIceCandidate, RtcIceCandidateInit, RtcIceConnectionState,
    RtcIceGatheringState, RtcPeerConnection, RtcPeerConnectionIceEvent, RtcSessionDescriptionInit,
};

use yew::callback::Callback;

type SingleArgClosure = Closure<dyn FnMut(JsValue)>;
type SingleArgJsFn = Box<dyn FnMut(JsValue)>;

const STUN_SERVER: &str = "stun:stun.l.google.com:19302";

#[derive(Clone, Debug, PartialEq)]
pub enum WebRtcMessage {
    Message(String),
    UpdateState(State),
    Reset,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IceOfferBundle {
    pub ice_candidates: Vec<IceCandidate>,
    pub offer: String,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ConnectionState {
    pub ice_gathering_state: Option<RtcIceGatheringState>,
    pub ice_connection_state: Option<RtcIceConnectionState>,
    pub message_channel_state: Option<RtcDataChannelState>,
    pub data_channel_state: Option<RtcDataChannelState>,
}

impl ConnectionState {
    pub fn new() -> ConnectionState {
        ConnectionState {
            ice_gathering_state: None,
            ice_connection_state: None,
            message_channel_state: None,
            data_channel_state: None,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum OfferError {
    InvalidBase64,
    InvalidString,
    SerializationError,
    InvalidOffer,
    //InvalidCandidate,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum State {
    Default,
    Server(ConnectionState),
    Client(ConnectionState),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IceCandidate {
    candidate: String,
    sdp_mid: String,
    sdp_m_line_index: u16,
}

pub struct WebRTCManager {
    callback: Callback<WebRtcMessage>,
    state: State,
    rtc_peer_connection: Option<RtcPeerConnection>,
    data_channel: Option<RtcDataChannel>,
    exit_offer_or_answer_early: bool,
    ice_candidates: Vec<IceCandidate>,
    offer: Option<String>,
}

impl WebRTCManager {
    pub fn new(callback: Callback<WebRtcMessage>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(WebRTCManager {
            callback,
            state: State::Default,
            rtc_peer_connection: None,
            data_channel: None,
            ice_candidates: Vec::new(),
            offer: None,
            exit_offer_or_answer_early: false,
        }))
    }

    pub fn send_message(&self, message_content: &str) {
        self.data_channel
            .as_ref()
            .expect("must have a data channel")
            .send_with_str(message_content)
            .expect("channel is open");
    }

    fn get_state(&self) -> State {
        self.state.clone()
    }

    pub fn set_state(&mut self, new_state: State) {
        self.state = new_state;
    }

    pub fn get_offer(&self) -> Option<String> {
        self.offer.clone()
    }

    fn get_ice_candidates(&self) -> Vec<IceCandidate> {
        self.ice_candidates.clone()
    }

    pub fn validate_offer(
        web_rtc_manager: Rc<RefCell<WebRTCManager>>,
        str: &str,
    ) -> Result<(), OfferError> {
        let ice_offer_bundle = Self::parse_base64_str_to_connection(str).map_err(|e| e)?;

        let remote_description_js_value: JsValue =
            JSON::parse(&ice_offer_bundle.offer).expect("Expected valid json");
        let remote_description =
            remote_description_js_value.unchecked_into::<RtcSessionDescriptionInit>();

        let set_remote_description_exception_handler = Self::create_exception_handler(web_rtc_manager.clone());
        let create_answer_closure = Self::create_answer_closure(web_rtc_manager.clone(), ice_offer_bundle);

        let _promise = web_rtc_manager
            .borrow()
            .rtc_peer_connection
            .as_ref()
            .unwrap()
            .set_remote_description(&remote_description)
            .catch(&set_remote_description_exception_handler)
            .then(&create_answer_closure);

        create_answer_closure.forget();
        Ok(())
    }

    fn create_answer_closure(
        web_rtc_manager: Rc<RefCell<WebRTCManager>>,
        ice_offer_bundle: IceOfferBundle,
    ) -> SingleArgClosure {
        let create_answer_function: SingleArgJsFn = Box::new(move |_: JsValue| {
            let set_local_description_closure = Self::create_set_local_description_closure(web_rtc_manager.clone());
            let create_answer_exception_handler = Self::create_exception_handler(web_rtc_manager.clone());
            let set_candidates_closure = Self::create_set_candidates_closure(web_rtc_manager.clone(), ice_offer_bundle.clone());
    
            let future = web_rtc_manager
                .borrow()
                .rtc_peer_connection
                .as_ref()
                .unwrap()
                .create_answer();
    
            // Chain the asynchronous operations
            _ = JsFuture::from(
                future
                    .then(&set_local_description_closure)
                    .catch(&create_answer_exception_handler)
                    .then(&set_candidates_closure)
            );
    
            set_candidates_closure.forget();
            set_local_description_closure.forget();
        });
    
        Closure::wrap(create_answer_function)
    }    

    fn create_set_candidates_closure(
        web_rtc_manager: Rc<RefCell<WebRTCManager>>,
        ice_offer_bundle: IceOfferBundle,
    ) -> SingleArgClosure {
        let set_candidates_function: SingleArgJsFn = Box::new(move |_: JsValue| {
            WebRTCManager::set_candidates(web_rtc_manager.clone(), &ice_offer_bundle);
        });
        Closure::wrap(set_candidates_function)
    }

    fn create_set_local_description_closure(
        web_rtc_manager: Rc<RefCell<WebRTCManager>>,
    ) -> SingleArgClosure {
        let function: SingleArgJsFn = Box::new(move |answer: JsValue| {
            let answer = answer.unchecked_into::<RtcSessionDescriptionInit>();
            let _promise = web_rtc_manager
                .borrow()
                .rtc_peer_connection
                .as_ref()
                .unwrap()
                .set_local_description(&answer);
            web_rtc_manager.borrow_mut().offer =
                Some(String::from(JSON::stringify(&answer).unwrap()));
        });
        Closure::wrap(function)
    }

    fn create_exception_handler(
        web_rtc_manager: Rc<RefCell<WebRTCManager>>,
    ) -> SingleArgClosure {
        let function: SingleArgJsFn = Box::new(move |_send_channel: JsValue| {
            web_rtc_manager.borrow_mut().exit_offer_or_answer_early = true;
        });
        Closure::wrap(function)
    }


    pub fn validate_answer(
        web_rtc_manager: Rc<RefCell<WebRTCManager>>,
        str: &str,
    ) -> Result<(), OfferError> {
        let ice_offer_bundle = Self::parse_base64_str_to_connection(str)?;
    
        let remote_description_js_value = JSON::parse(&ice_offer_bundle.offer).expect("Expected valid json");
        let remote_description = remote_description_js_value.unchecked_into::<RtcSessionDescriptionInit>();
    
        let exception_handler = Self::get_exception_handler(
            web_rtc_manager.clone(),
            "Promise set_remote_description encountered an exception. See console for details".to_string(),
        );
    
        let set_candidates_closure = Self::get_set_candidates_closure(web_rtc_manager.clone(), ice_offer_bundle);
    
        let _ = web_rtc_manager.borrow().rtc_peer_connection.as_ref().unwrap()
            .set_remote_description(&remote_description)
            .catch(&exception_handler)
            .then(&set_candidates_closure);
    
        set_candidates_closure.forget();
    
        Ok(())
    }

    fn get_set_candidates_closure(
        web_rtc_manager: Rc<RefCell<WebRTCManager>>,
        ice_offer_bundle: IceOfferBundle,
    ) -> SingleArgClosure {
        let ice_offer_bundle = Rc::new(ice_offer_bundle);
        let web_rtc_manager_rc_clone = web_rtc_manager.clone();
    
        Closure::wrap(Box::new(move |_: JsValue| {
            Self::set_candidates(web_rtc_manager_rc_clone.clone(), &*ice_offer_bundle);
        }) as SingleArgJsFn)
    }

    pub fn start_web_rtc(web_rtc_manager: Rc<RefCell<Self>>) -> Result<(), JsValue> {
        let rtc_peer_connection = Self::initialize_rtc_peer_connection()?;
        let state = web_rtc_manager.borrow().state.clone();
        match state {
            State::Server(_) => Self::setup_as_server(web_rtc_manager.clone(), rtc_peer_connection.clone()),
            State::Client(_) => Self::setup_as_client(web_rtc_manager.clone(), rtc_peer_connection.clone()),
            _ => panic!("Not implemented"),
        }
        Self::setup_rtc_event_listeners(web_rtc_manager, rtc_peer_connection)
    }
    
    fn initialize_rtc_peer_connection() -> Result<RtcPeerConnection, JsValue> {
        let ice_servers = Array::new();
        let server_entry = Object::new();
        Reflect::set(&server_entry, &"urls".into(), &STUN_SERVER.into())?;
        ice_servers.push(&*server_entry);
        let mut rtc_configuration = RtcConfiguration::new();
        rtc_configuration.ice_servers(&ice_servers);
        RtcPeerConnection::new_with_configuration(&rtc_configuration)
    }
    
    fn setup_as_server(web_rtc_manager: Rc<RefCell<Self>>, rtc_peer_connection: RtcPeerConnection) {
        let mut data_channel_init = RtcDataChannelInit::new();
        data_channel_init.ordered(true);
        let data_channel = rtc_peer_connection.create_data_channel_with_data_channel_dict("sendChannel", &data_channel_init);
        web_rtc_manager.borrow_mut().data_channel = Self::set_data_channel(web_rtc_manager.clone(), data_channel);
    
        let create_offer_closure = Self::create_offer_closure(web_rtc_manager.clone());
        let _create_offer_promise = rtc_peer_connection.create_offer().then(&create_offer_closure);
        create_offer_closure.forget();
    }
    
    fn setup_as_client(web_rtc_manager: Rc<RefCell<Self>>, rtc_peer_connection: RtcPeerConnection) {
        let on_data_channel_closure = Self::on_data_channel_closure(web_rtc_manager);
        rtc_peer_connection.set_ondatachannel(Some(on_data_channel_closure.as_ref().unchecked_ref()));
        on_data_channel_closure.forget();
    }
    
    fn setup_rtc_event_listeners(web_rtc_manager: Rc<RefCell<Self>>, rtc_peer_connection: RtcPeerConnection) -> Result<(), JsValue> {
        let on_ice_candidate_closure = Self::on_ice_candidate_closure(web_rtc_manager.clone());
        let on_ice_connection_state_change_closure = Self::get_on_ice_connection_state_change_closure(web_rtc_manager.clone());
        let on_ice_gathering_state_change_closure = Self::get_on_ice_gathering_state_change_closure(web_rtc_manager.clone());
    
        rtc_peer_connection.set_onicecandidate(Some(on_ice_candidate_closure.as_ref().unchecked_ref()));
        rtc_peer_connection.set_oniceconnectionstatechange(Some(on_ice_connection_state_change_closure.as_ref().unchecked_ref()));
        rtc_peer_connection.set_onicegatheringstatechange(Some(on_ice_gathering_state_change_closure.as_ref().unchecked_ref()));
    
        web_rtc_manager.borrow_mut().rtc_peer_connection = Some(rtc_peer_connection);
        on_ice_candidate_closure.forget();
        on_ice_connection_state_change_closure.forget();
        on_ice_gathering_state_change_closure.forget();
        Ok(())
    }
    
    fn create_offer_closure(web_rtc_manager: Rc<RefCell<Self>>) -> SingleArgClosure {
        let function: SingleArgJsFn = Box::new(move |offer: JsValue| {
            let rtc_session_description: RtcSessionDescriptionInit = offer.unchecked_into();
            let description_string = JSON::stringify(&rtc_session_description).unwrap();
            web_rtc_manager.borrow_mut().offer = Some(description_string.into());
    
            let set_local_description_exception_handler = Self::get_exception_handler(web_rtc_manager.clone(), "set_local_description closure has encountered an exception".into());
            let _promise = web_rtc_manager.borrow_mut().rtc_peer_connection.as_ref().unwrap().set_local_description(&rtc_session_description).catch(&set_local_description_exception_handler);
        });
        Closure::wrap(function)
    }
    
    fn on_data_channel_closure(web_rtc_manager: Rc<RefCell<Self>>) -> SingleArgClosure {
        let function: SingleArgJsFn = Box::new(move |data_channel_event: JsValue| {
            let event = data_channel_event.unchecked_into::<RtcDataChannelEvent>();
            let data_channel = event.channel();
            web_rtc_manager.borrow_mut().data_channel = Self::set_data_channel(web_rtc_manager.clone(), data_channel);
        });
        Closure::wrap(function)
    }
    
    fn on_ice_candidate_closure(web_rtc_manager: Rc<RefCell<Self>>) -> SingleArgClosure {
        let function: SingleArgJsFn = Box::new(move |ice_connection_event: JsValue| {
            let ice_connection_event_obj = ice_connection_event.unchecked_into::<RtcPeerConnectionIceEvent>();
            if let Some(candidate) = ice_connection_event_obj.candidate() {
                let candidate_str = candidate.candidate();
                if !candidate_str.is_empty() {
                    let saved_candidate = IceCandidate {
                        candidate: candidate_str,
                        sdp_mid: candidate.sdp_mid().unwrap(),
                        sdp_m_line_index: candidate.sdp_m_line_index().unwrap(),
                    };
                    web_rtc_manager.borrow_mut().ice_candidates.push(saved_candidate);
                }
            }
        });
        Closure::wrap(function)
    }
    
    pub fn create_encoded_offer(&self) -> String {
        let ice_offer_bundle = IceOfferBundle {
            offer: self.get_offer().expect("no offer yet"),
            ice_candidates: self.get_ice_candidates(),
        };

        let serialized: String = serde_json::to_string(&ice_offer_bundle).unwrap();

        general_purpose::STANDARD.encode(serialized)
    }

    fn set_candidates(
        web_rtc_manager: Rc<RefCell<Self>>,
        ice_offer_bundle: &IceOfferBundle,
    ) {
        let manager = web_rtc_manager.borrow();
    
        if manager.exit_offer_or_answer_early {
            return;
        }
    
        let rtc_peer_connection = match manager.rtc_peer_connection.as_ref() {
            Some(connection) => connection,
            None => return, // If the connection doesn't exist, exit early.
        };
    
        let exception_handler = Self::get_exception_handler(
            web_rtc_manager.clone(),
            "add_candidate closure has encountered an exception".into(),
        );
    
        for candidate in &ice_offer_bundle.ice_candidates {
            let ice_candidate = Self::create_ice_candidate(candidate);
            
            let _promise = rtc_peer_connection
                .add_ice_candidate_with_opt_rtc_ice_candidate(Some(&ice_candidate))
                .catch(&exception_handler);
        }
    }

    fn create_ice_candidate(candidate_data: &IceCandidate) -> RtcIceCandidate {
        let mut ice_candidate_init = RtcIceCandidateInit::new("");

        ice_candidate_init.candidate(&candidate_data.candidate);
        ice_candidate_init.sdp_mid(Some(&candidate_data.sdp_mid));
        ice_candidate_init.sdp_m_line_index(Some(candidate_data.sdp_m_line_index));

        RtcIceCandidate::new(&ice_candidate_init).expect("valid candidate")
    }

    fn parse_base64_str_to_connection(str: &str) -> std::result::Result<IceOfferBundle, OfferError> {
        general_purpose::STANDARD.decode(str)
            .map_err(|_| OfferError::InvalidBase64)
            .and_then(|decoded| {
                let decoded_str = str::from_utf8(&decoded).map(|s| s.to_owned());
                decoded_str.map_err(|_| OfferError::InvalidString)
            })
            .and_then(|decoded_string| {
                serde_json::from_str::<IceOfferBundle>(&decoded_string)
                    .map_err(|_| OfferError::SerializationError)
            })
            .and_then(|ice_offer_bundle| {
                JSON::parse(&ice_offer_bundle.offer)
                    .map_err(|_| OfferError::InvalidOffer)
                    .map(|_| ice_offer_bundle)
            })
    }

    fn get_channel_status_change_closure(
        web_rtc_manager: Rc<RefCell<Self>>,
    ) -> SingleArgClosure {
        Self::get_closure(web_rtc_manager, |connection_state, inner| {
            connection_state.data_channel_state = inner.data_channel.as_ref().map(|dc| dc.ready_state());
        })
    }

    fn get_on_data_closure(web_rtc_manager: Rc<RefCell<Self>>) -> SingleArgClosure {
        Closure::wrap(Box::new(move |arg: JsValue| {
            let message_event = arg.unchecked_into::<web_sys::MessageEvent>();
            let msg_content: String = message_event.data().as_string().unwrap();
            web_rtc_manager
                .borrow()
                .callback
                .emit(WebRtcMessage::Message(msg_content));
        }) as SingleArgJsFn)
    }

    fn get_on_ice_connection_state_change_closure(
        web_rtc_manager: Rc<RefCell<Self>>,
    ) -> SingleArgClosure {
        Self::get_closure(web_rtc_manager, |connection_state, inner| {
            connection_state.ice_connection_state = Some(inner.rtc_peer_connection.as_ref().unwrap().ice_connection_state());
        })
    }

    fn get_on_ice_gathering_state_change_closure(
        web_rtc_manager: Rc<RefCell<Self>>,
    ) -> SingleArgClosure {
        Self::get_closure(web_rtc_manager, |connection_state, inner| {
            connection_state.ice_gathering_state = Some(inner.rtc_peer_connection.as_ref().unwrap().ice_gathering_state());
        })
    }

    fn get_closure<F>(
        web_rtc_manager: Rc<RefCell<Self>>,
        mut action: F,
    ) -> SingleArgClosure
    where
        F: 'static + FnMut(&mut ConnectionState, &WebRTCManager),
    {
        Closure::wrap(Box::new(move |_event: JsValue| {
            let new_state = {
                let inner = web_rtc_manager.borrow();
                let mut temp_state = inner.get_state();
            
                match &mut temp_state {
                    State::Server(connection_state) | State::Client(connection_state) => {
                        action(connection_state, &*inner);
                    }
                    _ => {}
                }
            
                temp_state
            }; 
            
            web_rtc_manager.borrow_mut().set_state(new_state);
            let web_rtc_state = web_rtc_manager.borrow().get_state();
            web_rtc_manager
                .borrow()
                .callback
                .emit(WebRtcMessage::UpdateState(web_rtc_state));
        }) as SingleArgJsFn)
    }
    
    fn get_exception_handler(
        web_rtc_manager: Rc<RefCell<WebRTCManager>>,
        message: String,
    ) -> SingleArgClosure {
        let web_rtc_manager_rc_clone = web_rtc_manager.clone();
        Closure::wrap(Box::new(move |a: JsValue| {
            web_rtc_manager_rc_clone.borrow_mut().exit_offer_or_answer_early = true;
    
            console::log_1(&"Exception handler !".into());
            console::log_1(&a);
    
            web_sys::Window::alert_with_message(&web_sys::window().unwrap(), &message).expect("alert should work");
            web_rtc_manager_rc_clone.borrow().callback.emit(WebRtcMessage::Reset);
        }) as SingleArgJsFn)
    }

    fn set_data_channel(web_rtc_manager: Rc<RefCell<WebRTCManager>>, data_channel: RtcDataChannel) -> Option<RtcDataChannel> {
        let channel_status_change_closure = WebRTCManager::get_channel_status_change_closure(web_rtc_manager.clone());
        let channel_status_change_ref = channel_status_change_closure.as_ref().unchecked_ref();
        data_channel.set_onopen(Some(channel_status_change_ref));
        data_channel.set_onclose(Some(channel_status_change_ref));
    
        let on_data_closure = WebRTCManager::get_on_data_closure(web_rtc_manager);
        let on_data_ref = on_data_closure.as_ref().unchecked_ref();
        data_channel.set_onmessage(Some(on_data_ref));
    
        channel_status_change_closure.forget();
        on_data_closure.forget();
        Some(data_channel)
    }
    
}
