use typed_arena::Arena;

struct RetEntry {
    u8: variation
}
struct ExternEntry {
    
}

struct ProgramArena {
    entries: Arena<ArenaEntry>,
    groups: Arena<Group>,
    externs: Arena<string>
}

impl ProgramArena {
    pub fn add_extern(&mut self, ent:)
}