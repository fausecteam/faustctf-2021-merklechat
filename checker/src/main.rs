use checkerlib::{Checker, Error, Context, CheckerResult};


struct MerkleChecker {
    context: Context
}

impl MerkleChecker {
    fn new(context:Context) -> Self {
        Self { context: context }
    }
}

impl Checker for MerkleChecker {

    fn place_flag(&mut self) -> Result<(), Error> {
        self.context.store_data("test", &String::from("test"))?;
        Ok(())
    }

    fn check_flag(&mut self, tick:u32) -> Result<(), Error> {
        let _s:String = self.context.load_data("test")?;
        Ok(())
    }

    fn check_service(&mut self) -> Result<(), Error> {
        Err(Error::CheckerError(CheckerResult::Faulty))
    }
}


pub fn main() {
    checkerlib::run_check(MerkleChecker::new);
}
