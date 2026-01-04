pub enum PortEvent {
    RecvData { data: Vec<u8> },
    SendData { data: Vec<u8> },
    Closed { port_name: String }, // port name
}
