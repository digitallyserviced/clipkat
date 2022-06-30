
tonic::include_proto!("manager");
tonic::include_proto!("monitor");

impl From<ClipboardType> for crate::ClipboardType {
    fn from(t: ClipboardType) -> crate::ClipboardType {
        match t {
            ClipboardType::Clipboard => crate::ClipboardType::Clipboard,
            ClipboardType::Primary => crate::ClipboardType::Primary,
        }
    }
}

impl From<crate::ClipboardType> for ClipboardType {
    fn from(t: crate::ClipboardType) -> ClipboardType {
        match t {
            crate::ClipboardType::Clipboard => ClipboardType::Clipboard,
            crate::ClipboardType::Primary => ClipboardType::Primary,
        }
    }
}

impl From<crate::ClipboardType> for i32 {
    fn from(t: crate::ClipboardType) -> i32 { t as i32 }
}
impl From<ClipboardData> for crate::ClipboardData {
    fn from(data: ClipboardData) -> crate::ClipboardData {
        let timestamp = std::time::UNIX_EPOCH
                    .checked_add(std::time::Duration::from_millis(data.timestamp))
                    .unwrap_or_else(std::time::SystemTime::now);
        crate::ClipboardData {
            id: data.id as u64,
            data: data.data,
            size: data.size as usize,
            clipboard_type: data.clipboard_type.into(),
            timestamp: timestamp,
        }
    }
}

impl From<crate::ClipboardData> for ClipboardData {
    fn from(data: crate::ClipboardData) -> ClipboardData {
        ClipboardData {
            id: data.id as u64,
            data: data.data,
            size: data.size as u64,
            clipboard_type: data.clipboard_type.into(),
            timestamp: data
                .timestamp
                .duration_since(std::time::UNIX_EPOCH)
                .expect("duration since")
                .as_millis() as u64,
        }
    }
}

impl From<MonitorState> for crate::MonitorState {
    fn from(state: MonitorState) -> crate::MonitorState {
        match state {
            MonitorState::Enabled => crate::MonitorState::Enabled,
            MonitorState::Disabled => crate::MonitorState::Disabled,
        }
    }
}

impl Into<MonitorState> for crate::MonitorState {
    fn into(self) -> MonitorState {
        match self {
            crate::MonitorState::Enabled => MonitorState::Enabled,
            crate::MonitorState::Disabled => MonitorState::Disabled,
        }
    }
}

impl From<crate::MonitorState> for i32 {
    fn from(state: crate::MonitorState) -> i32 { state as i32 }
}
