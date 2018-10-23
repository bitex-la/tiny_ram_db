error_chain! {
    errors {
        RecordNotFound(t: String) {
            description("Record not found")
            display("Record not found: '{}'", t)
        }
    }

    foreign_links {
      PoisonError(std::sync::PoisonError)
    }
}
