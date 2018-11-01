use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

const ADDRESS_SPACE: usize = 65536;

pub trait Reader {
    fn read(&mut self, address: u16) -> u8;
}

pub trait Writer {
    fn write(&mut self, address: u16, byte: u8);
}

pub trait ReadWriter : Reader + Writer {}
impl<T: Reader + Writer> ReadWriter for T {}

pub struct Manager {
    modules: VecDeque<Module>,
}

pub fn new() -> Manager {
    let ram = Rc::new(RefCell::new(RAM::new()));
    let module = Module{
        delegate: ram,
        start_addr: 0,
        end_addr: (ADDRESS_SPACE-1) as u16,
    };

    let mut modules = VecDeque::new();
    modules.push_back(module);

    Manager{ modules }
}

impl Reader for Manager {
    fn read(&mut self, address: u16) -> u8 {
        let module = self.find_module(address).unwrap();
        return module.delegate.borrow_mut().read(address);
    }
}

impl Writer for Manager {
    fn write(&mut self, address: u16, byte: u8) {
        let module = self.find_module(address).unwrap();
        return module.delegate.borrow_mut().write(address, byte);
    }
}

impl Manager {
    pub fn mount(&mut self, delegate: Rc<RefCell<ReadWriter>>, start_addr: u16, end_addr: u16) {
        if end_addr as usize >= ADDRESS_SPACE {
            panic!("Mount end_addr is out of bounds: {} >= {}", end_addr as usize, ADDRESS_SPACE);
        }

        let module = Module{ delegate, start_addr, end_addr };

        self.modules.push_front(module)
    }

    fn find_module(&mut self, addr: u16) -> Option<&mut Module> {
        for module in self.modules.iter_mut() {
            if module.start_addr <= addr && module.end_addr >= addr {
                return Some(module);
            }
        }
        return None;
    }

    pub fn debug_print(&mut self, start_addr: u16, num_bytes: u16) {
        let end_addr = start_addr - 1 + num_bytes;
        print!("MEMORY [{:X}..{:X}]:", start_addr, end_addr);
        for ix in 0..num_bytes {
            print!(" {:X}", self.read(start_addr + ix));
        }
        println!();
    }
}

struct Module {
    delegate: Rc<RefCell<ReadWriter>>,
    start_addr: u16,
    end_addr: u16,
}

pub struct RAM {
    memory: [u8; ADDRESS_SPACE],
}

impl Reader for RAM {
    fn read(&mut self, address: u16) -> u8 {
        self.memory[address as usize]
    }
}

impl Writer for RAM {
    fn write(&mut self, address: u16, byte: u8) {
        self.memory[address as usize] = byte
    }
}

impl RAM {
    pub fn new() -> RAM {
        RAM{
            memory: [0; ADDRESS_SPACE],
        }
    }

    pub fn debug_print(&self, start_addr: u16, num_bytes: u16) {
        let end_addr = start_addr - 1 + num_bytes;
        println!("RAM [{:X}..{:X}]: {:?}", start_addr, end_addr, &self.memory[(start_addr as usize) .. (end_addr as usize)]);
    }
}

#[test]
fn test_get_and_set() {
    let mut ram = new();
    ram.write(1234, 23);
    assert_eq!(ram.read(1234), 23);
}
