use std::collections::HashMap;

use crate::tac::{tac_instr::TacInstr, Identifier};

use super::Location;

pub struct RegisterAllocator {
    ident_to_loc_map: HashMap<Identifier, Location>,
}

impl RegisterAllocator {
    pub fn new(tac_instrs: &Vec<TacInstr>) -> (Self, usize) {
        let mut set_of_temporaries: Vec<Identifier> = Vec::new();
        // let mut set_of_mem_chunks = Vec<

        for instr in tac_instrs {
            for ident in instr.get_read_identifiers() {
                if !set_of_temporaries.contains(&ident) {
                    eprintln!(
                        "warning: found read from temporary {:?} with writing first.",
                        ident
                    );
                }
            }
            if let Some(ident) = instr.get_written_identifier() {
                if !set_of_temporaries.contains(&ident) {
                    set_of_temporaries.push(ident);
                }
            }
        }

        let mut map = HashMap::new();

        let mut bytes_needed = 0;

        for identifier in &set_of_temporaries {
            bytes_needed += identifier.get_num_bytes();
            map.insert(*identifier, Location::Mem(bytes_needed));
        }

        (
            RegisterAllocator {
                ident_to_loc_map: map,
            },
            bytes_needed,
        )
    }

    pub fn get_location(&self, temporary: Identifier) -> Location {
        return *self.ident_to_loc_map.get(&temporary).unwrap();
    }
}
