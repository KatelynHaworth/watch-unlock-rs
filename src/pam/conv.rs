// TODO: Upstream this to pam-rs

use pam::{
    get_item, PamError, PamHandle, PamItemType, PamMessage, PamMessageStyle, PamResponse,
    PamResult, PamReturnCode,
};
use std::ffi::{c_int, CStr, CString};
use std::ptr;

#[repr(C)]
pub struct Inner {
    conv: extern "C" fn(
        num_msg: c_int,
        pam_message: &&PamMessage,
        pam_response: &mut *const PamResponse,
        appdata_ptr: *const libc::c_void,
    ) -> PamReturnCode,
    appdata_ptr: *const libc::c_void,
}

pub struct ClientConv<'a>(&'a Inner);

#[allow(unused)]
impl ClientConv<'_> {
    pub fn try_from(handle: &PamHandle) -> Result<Self, PamError> {
        unsafe {
            let ptr: *const libc::c_void = get_item(handle, PamItemType::Conv)?;
            let typed_ptr = ptr.cast::<Inner>();
            let data: &Inner = &*typed_ptr;

            #[allow(clippy::borrow_deref_ref)]
            Ok(Self(&*data))
        }
    }

    pub fn prompt_echo(&self, msg: &CStr) -> Result<CString, ()> {
        self.prompt(true, msg)
    }

    pub fn prompt_blind(&self, msg: &CStr) -> Result<CString, ()> {
        self.prompt(false, msg)
    }

    fn prompt(&self, echo: bool, msg: &CStr) -> Result<CString, ()> {
        let style = if echo {
            PamMessageStyle::Prompt_Echo_On
        } else {
            PamMessageStyle::Prompt_Echo_Off
        };

        match self.send(style, msg) {
            Ok(Some(resp)) => Ok(CString::from(resp)),
            Ok(None) => Ok(CString::new("").unwrap()),
            Err(_) => Err(()),
        }
    }

    pub fn info(&self, msg: &CStr) {
        let _ = self.send(PamMessageStyle::Text_Info, msg);
    }

    pub fn error(&self, msg: &CStr) {
        let _ = self.send(PamMessageStyle::Error_Msg, msg);
    }

    fn send(&self, style: PamMessageStyle, msg: &CStr) -> PamResult<Option<&CStr>> {
        let mut resp_ptr: *const PamResponse = ptr::null();
        let msg = PamMessage {
            msg_style: style as c_int,
            msg: msg.as_ptr(),
        };

        let ret = (self.0.conv)(1, &&msg, &mut resp_ptr, self.0.appdata_ptr);

        if ret == PamReturnCode::Success {
            let response = unsafe { (*resp_ptr).resp };
            if response.is_null() {
                Ok(None)
            } else {
                Ok(Some(unsafe { CStr::from_ptr(response) }))
            }
        } else {
            Err(PamError::from(ret))
        }
    }
}
