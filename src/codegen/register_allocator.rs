use std::collections::HashMap;

use crate::tac::{tac_instr::TacInstr, Identifier};

use super::Location;

pub struct RegisterAllocator {
    ident_to_loc_map: HashMap<Identifier, Location>,
    ident_to_init_val_map: HashMap<Identifier, usize>, // here the usize represents an offset, in bytes, from rbp.
}

impl RegisterAllocator {
    pub fn new(tac_instrs: &Vec<TacInstr>) -> (Self, usize) {
        let mut set_of_temporaries: Vec<Identifier> = Vec::new();

        for instr in tac_instrs {
            for ident in instr.get_read_identifiers() {
                if !set_of_temporaries.contains(&ident) {
                    eprintln!(
                        "warning: found read from temporary {:?} wit writing first.",
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

        let mut ident_to_loc_map = HashMap::new();

        let mut bytes_needed = 0;

        for identifier in &set_of_temporaries {
            bytes_needed += identifier.get_num_bytes();
            ident_to_loc_map.insert(*identifier, Location::Mem(bytes_needed));
        }

        let mut ident_to_init_val_map: HashMap<Identifier, usize> = HashMap::new();
        for instr in tac_instrs {
            if let TacInstr::MemChunk(ptr_ident, chunk_size, _) = instr {
                bytes_needed += chunk_size;
                ident_to_init_val_map.insert(*ptr_ident, bytes_needed);
            }
        }

        (
            RegisterAllocator {
                ident_to_loc_map,
                ident_to_init_val_map,
            },
            bytes_needed,
        )
    }

    pub fn get_location(&self, temporary: Identifier) -> Location {
        return *self.ident_to_loc_map.get(&temporary).unwrap();
    }

    pub fn get_ptr_init_val(&self, temporary: Identifier) -> usize {
        // expects the temporary to be initialized with an MemChunk() TAC statement
        // returns a usize representing an offset from rbp of the chunk of allocated stack memory
        return *self.ident_to_init_val_map.get(&temporary).unwrap();
    }
}
