use crate::{
    makepad_live_id::LiveId,
    makepad_code_editor::text::{Position, Length},
    makepad_micro_serde::{SerBin, DeBin, DeBinErr},
};


#[derive(PartialEq, Clone, Copy, Debug, SerBin, DeBin)]
pub struct BuildCmdId(pub u64);

#[cfg(not(target_os="windows"))]
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuildTarget {
    Release,
    Debug,
    ReleaseStudio,
    DebugStudio,
    Profiler,
    IosSim{org:String, app:String},
    IosDevice{org:String, app:String},
    Android,
    WebAssembly
}

#[cfg(not(target_os="windows"))]
impl BuildTarget {
    pub fn runs_in_studio(&self)->bool{
        match self{
            Self::ReleaseStudio=>true,
            Self::DebugStudio=>true,
            _=>false
        }
    }
    
    pub const RELEASE_STUDIO:u64 = 0;
    pub const DEBUG_STUDIO:u64 = 1;
    pub const RELEASE:u64 = 2;
    pub const DEBUG:u64 = 3;
    pub const PROFILER:u64 = 4;
    pub const IOS_SIM:u64 = 5;
    pub const IOS_DEVICE:u64 = 6;
    pub const ANDROID:u64 = 7;
    pub const WEBASSEMBLY:u64 = 8;
    pub fn len() -> u64 {9}
    pub fn name(idx: u64) -> &'static str {
        match idx {
            Self::RELEASE_STUDIO=> "Release Studio",
            Self::DEBUG_STUDIO=> "Debug Studio",
            Self::RELEASE=> "Release",
            Self::DEBUG=> "Debug",
            Self::PROFILER=> "Profiler",
            Self::IOS_SIM=> "iOS Simulator",
            Self::IOS_DEVICE=> "iOS Device",
            Self::ANDROID=> "Android",
            Self::WEBASSEMBLY=> "WebAssembly",
            _=>"Unknown"
        }
    }
    pub fn id(&self) -> u64 {
        match self {
            Self::ReleaseStudio=>Self::RELEASE_STUDIO,
            Self::DebugStudio=>Self::DEBUG_STUDIO,
            Self::Release=>Self::RELEASE,
            Self::Debug=>Self::DEBUG,
            Self::Profiler=>Self::PROFILER,
            Self::IosSim{..}=>Self::IOS_SIM,
            Self::IosDevice{..}=>Self::IOS_DEVICE,
            Self::Android=>Self::ANDROID,
            Self::WebAssembly=>Self::WEBASSEMBLY
        }
    }
}

#[cfg(target_os="windows")]
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuildTarget {
    Release,
    Debug,
    Profiler,
    IosSim{org:String, app:String},
    IosDevice{org:String, app:String},
    Android,
    WebAssembly
}

#[cfg(target_os="windows")]
impl BuildTarget {
    pub fn runs_in_studio(&self)->bool{
        match self{
            _=>false
        }
    }
    
    pub const RELEASE:u64 = 0;
    pub const DEBUG:u64 = 1;
    pub const PROFILER:u64 = 2;
    pub const IOS_SIM:u64 = 3;
    pub const IOS_DEVICE:u64 = 4;
    pub const ANDROID:u64 = 5;
    pub const WEBASSEMBLY:u64 = 6;
    pub fn len() -> u64 {7}
    pub fn name(idx: u64) -> &'static str {
        match idx {
            Self::RELEASE=> "Release",
            Self::DEBUG=> "Debug",
            Self::PROFILER=> "Profiler",
            Self::IOS_SIM=> "iOS Simulator",
            Self::IOS_DEVICE=> "iOS Device",
            Self::ANDROID=> "Android",
            Self::WEBASSEMBLY=> "WebAssembly",
            _=>"Unknown"
        }
    }
    pub fn id(&self) -> u64 {
        match self {
            Self::Release=>Self::RELEASE,
            Self::Debug=>Self::DEBUG,
            Self::Profiler=>Self::PROFILER,
            Self::IosSim{..}=>Self::IOS_SIM,
            Self::IosDevice{..}=>Self::IOS_DEVICE,
            Self::Android=>Self::ANDROID,
            Self::WebAssembly=>Self::WEBASSEMBLY
        }
    }
}


#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BuildProcess{
    pub binary: String,
    pub target: BuildTarget
}

impl BuildProcess{
    pub fn as_id(&self)->LiveId{
        LiveId::from_str(&self.binary).bytes_append(&self.target.id().to_be_bytes())
    }
}


#[derive(Clone, Debug)]
pub struct BuildCmdWrap {
    pub cmd_id: BuildCmdId,
    pub cmd: BuildCmd
}

impl BuildCmdId{
    pub fn wrap_msg(&self, item:LogItem)->LogItemWrap{
        LogItemWrap{
            cmd_id: *self,
            item,
        }
    }
}

#[derive(Clone, Debug)]
pub enum BuildCmd {
    Stop,
    Run(BuildProcess, String),
    HostToStdin(String)
}

#[derive(Clone)]
pub struct LogItemWrap {
    pub cmd_id: BuildCmdId,
    pub item: LogItem
}

#[derive(Clone, PartialEq, Eq, Copy, Debug, SerBin, DeBin)]
pub enum LogItemLevel{
    Warning,
    Error,
    Log,
    Wait,
    Panic,
}

#[derive(Clone, Debug)]
pub struct LogItemLocation{
    pub level: LogItemLevel,
    pub file_name: String,
    pub start: Position,
    pub length: Length,
    pub msg: String
}

#[derive(Clone, Debug)]
pub struct LogItemBare{
    pub level: LogItemLevel,
    pub line: String,
}

#[derive(Clone)]
pub enum LogItem {
    Bare(LogItemBare),
    Location(LogItemLocation),
    StdinToHost(String),
    AuxChanHostEndpointCreated(crate::makepad_platform::cx_stdin::aux_chan::HostEndpoint),
}
