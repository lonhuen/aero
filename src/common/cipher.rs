pub struct CipherText {
    // 128 bit * 4096 = 8 bit * 65536
    pub c0: [u8; 65536],
    pub c1: [u8; 65536],
}

impl CipherText {
    pub fn new() -> Self {
        Self {
            // just for test purpose
            //c0: [1u8; 65536],
            //c1: [1u8; 65536],
            c0: [0u8; 65536],
            c1: [0u8; 65536],
        }
    }
}
