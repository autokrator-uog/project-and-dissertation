use clap::ArgMatches;

pub struct Context {
    pub brokers: String,
    pub group: String,
    pub input: String,
    pub output: String
}

impl Context {
    pub fn new(args: ArgMatches) -> Context {
        let brokers = args.value_of("brokers").unwrap();
        let group = args.value_of("group").unwrap();
        let input = args.value_of("input-topic").unwrap();
        let output = args.value_of("output-topic").unwrap();

        Context {
            brokers: brokers.to_string(),
            group: group.to_string(),
            input: input.to_string(),
            output: output.to_string()
        }
    }
}
