pub(crate) struct Code {
    indent_space: u8,
    indent_level: u8,
    content: String
}

impl Code {
    pub(crate) fn new<F: Fn(&mut Code)>(indent_level: u8, indent_space: u8, build: F) -> Self {
        let mut code = Code { indent_level, indent_space, content: String::new() };
        build(&mut code);
        code
    }

    pub(crate) fn line<L: Into<String>>(&mut self, line: L) {
        self.content += &" ".repeat((self.indent_level * self.indent_space) as usize);
        self.content += &line.into();
        self.content += "\n";
    }

    pub(crate) fn empty_line(&mut self) {
        self.content += "\n";
    }

    pub(crate) fn block<S: Into<String>, F: Fn(&mut Code), E: Into<String>>(&mut self, start: S, build: F, end: E) {
        let code = Code::new(self.indent_level + 1, self.indent_space, build);
        self.content += &" ".repeat((self.indent_level * self.indent_space) as usize);
        self.content += &start.into();
        self.content += "\n";
        self.content += code.to_str();
        self.content += &" ".repeat((self.indent_level * self.indent_space) as usize);
        self.content += &end.into();
        self.content += "\n";
    }

    pub(crate) fn to_str(&self) -> &str {
        &self.content
    }

    pub(crate) fn to_string(&self) -> String {
        self.to_str().to_string()
    }
}