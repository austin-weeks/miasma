use std::borrow::Cow;

use bytes::Bytes;

pub fn as_bytes(cow: Cow<'static, str>) -> Bytes {
    match cow {
        Cow::Borrowed(s) => Bytes::from_static(s.as_bytes()),
        Cow::Owned(s) => Bytes::from(s.into_bytes()),
    }
}
