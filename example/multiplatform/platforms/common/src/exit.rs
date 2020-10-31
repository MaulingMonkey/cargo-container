pub fn errors()                     -> ! { std::process::exit(0xEE) } // EE = EE = Errors
pub fn warnings()                   -> ! { std::process::exit(0x33) } // 33 = WW = Warnings
pub fn command_not_implemented()    -> ! { std::process::exit(0xC1) } // C1 = CnI = Command Not Implemented
pub fn platform_not_implemented()   -> ! { std::process::exit(0x91) } // 91 = PnI = Platform Not Implemented
