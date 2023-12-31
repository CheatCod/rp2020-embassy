use super::PacketHeader;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct Cmd {
    pub cmd_code: u16,
    pub payload_len: u8,
    pub payload: [u8; 255],
}

impl Default for Cmd {
    fn default() -> Self {
        Self {
            cmd_code: 0,
            payload_len: 0,
            payload: [0u8; 255],
        }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
pub struct CmdSerial {
    pub ty: u8,
    pub cmd: Cmd,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
pub struct CmdPacket {
    pub header: PacketHeader,
    pub cmd_serial: CmdSerial,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct AclDataSerial {
    pub ty: u8,
    pub handle: u16,
    pub length: u16,
    pub acl_data: [u8; 1],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct AclDataPacket {
    pub header: PacketHeader,
    pub acl_data_serial: AclDataSerial,
}
