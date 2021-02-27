#![deny(unsafe_code)]

use std::{fmt::Display, ops::Range};

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{self, Snippet},
};

pub use annotate_snippets::snippet::AnnotationType;

fn source_line(source: &str, line_start: usize, line_end: usize) -> String {
    source.split("\n").collect::<Vec<_>>()[line_start - 1..line_end].join("\n").to_string()
}

#[derive(Debug, Default)]
pub struct SourceAnnotation {
    range: Option<Range<usize>>,
    label: Option<String>,
    kind: Option<AnnotationType>,
}

impl SourceAnnotation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_range(mut self, range: Range<usize>) -> Self {
        self.range = Some(range);

        self
    }

    pub fn set_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());

        self
    }

    pub fn set_kind(mut self, kind: AnnotationType) -> Self {
        self.kind = Some(kind);

        self
    }

    pub(crate) fn build(self) -> snippet::SourceAnnotation {
        let range = self.range.unwrap();

        snippet::SourceAnnotation {
            range: (range.start, range.end),
            label: self.label.unwrap_or_default(),
            annotation_type: self.kind.unwrap(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Slice {
    line_start: Option<usize>,
    line_end: Option<usize>,
    annotations: Vec<snippet::SourceAnnotation>,
}

impl Slice {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_line_start(mut self, line_start: usize) -> Self {
        self.line_start = Some(line_start);

        self
    }

    pub fn set_line_end(mut self, line_end: usize) -> Self {
        self.line_end = Some(line_end);

        self
    }

    pub fn push_annotation(mut self, source_annotation: SourceAnnotation) -> Self {
        self.annotations.push(source_annotation.build());

        self
    }

    pub(crate) fn build(&self, source: &str, origin: &str) -> snippet::Slice {
        snippet::Slice {
            source: source_line(source, self.line_start.unwrap(), self.line_end.unwrap_or(self.line_start.unwrap())),
            origin: Some(origin.to_string()),
            line_start: self.line_start.unwrap(),
            annotations: self.annotations.clone(),
            fold: true,
        }
    }
}

#[derive(Debug)]
pub struct Diagnostic(Snippet);

#[derive(Debug, Default)]
pub struct DiagnosticBuilder {
    kind: Option<AnnotationType>,
    message: Option<String>,
    code: Option<String>,
    source: Option<String>,
    origin: Option<String>,
    slices: Vec<snippet::Slice>,
}

impl DiagnosticBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_type(mut self, kind: AnnotationType) -> Self {
        self.kind = Some(kind);

        self
    }

    pub fn set_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());

        self
    }

    pub fn set_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());

        self
    }

    pub fn set_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());

        self
    }

    pub fn set_origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = Some(origin.into());

        self
    }

    pub fn push_slice(mut self, slice: Slice) -> Self {
        let origin = self.origin.as_ref().unwrap();
        let source = self.source.as_ref().unwrap();

        let slice = slice.build(source, origin);

        self.slices.push(slice);
        self
    }

    pub fn build(self) -> Diagnostic {
        assert!(self.message.is_some());
        assert!(self.kind.is_some());
        assert!(self.message.is_some());

        let title = snippet::Annotation {
            id: self.code,
            label: self.message,
            annotation_type: self.kind.unwrap(),
        };

        Diagnostic(Snippet {
            title: Some(title),
            footer: vec![],
            slices: self.slices,
            opt: FormatOptions {
                color: true,
                anonymized_line_numbers: false,
            },
        })
    }
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dl = DisplayList::from(self.0.clone());

        write!(f, "{}", dl)
    }
}
