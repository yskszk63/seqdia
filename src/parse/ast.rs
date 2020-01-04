use std::iter::IntoIterator;
use std::iter::Iterator;
use std::slice;

use super::visitor::Visitor;

#[derive(Debug, Clone)]
pub(crate) struct Document<'i>(Vec<Statement<'i>>);

impl<'i> Document<'i> {
    pub(crate) fn accept<V, C, O>(&self, visitor: &V, cx: &mut C) -> O
    where
        V: Visitor<'i, Context = C, Output = O>,
    {
        visitor.visit_document(self, cx)
    }

    pub fn iter<'s>(&'s self) -> DocumentIter<'i, 's> {
        DocumentIter(self.0.iter())
    }
}

impl<'i, 's> IntoIterator for &'s Document<'i> {
    type Item = &'s Statement<'i>;
    type IntoIter = DocumentIter<'i, 's>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug)]
pub(crate) struct DocumentIter<'i, 's>(slice::Iter<'s, Statement<'i>>);

impl<'i, 's> Iterator for DocumentIter<'i, 's> {
    type Item = &'s Statement<'i>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'i> From<Vec<Statement<'i>>> for Document<'i> {
    fn from(v: Vec<Statement<'i>>) -> Document {
        Document(v)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Statement<'i> {
    Title(Title<'i>),
    Signal(Signal<'i>),
    Participant(Participant<'i>),
    Note(Note<'i>),
}

impl<'i> Statement<'i> {
    pub(crate) fn accept<V, C, O>(&self, visitor: &V, cx: &mut C) -> O
    where
        V: Visitor<'i, Context = C, Output = O>,
    {
        visitor.visit_statement(self, cx)
    }
}

impl<'i> From<Title<'i>> for Statement<'i> {
    fn from(v: Title) -> Statement {
        Statement::Title(v)
    }
}

impl<'i> From<Signal<'i>> for Statement<'i> {
    fn from(v: Signal) -> Statement {
        Statement::Signal(v)
    }
}

impl<'i> From<Participant<'i>> for Statement<'i> {
    fn from(v: Participant) -> Statement {
        Statement::Participant(v)
    }
}

impl<'i> From<Note<'i>> for Statement<'i> {
    fn from(v: Note) -> Statement {
        Statement::Note(v)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Title<'i>(&'i str);

impl<'i> Title<'i> {
    pub(crate) fn new(v: &'i str) -> Title {
        Title(v)
    }

    pub(crate) fn accept<V, C, O>(&self, visitor: &V, cx: &mut C) -> O
    where
        V: Visitor<'i, Context = C, Output = O>,
    {
        visitor.visit_title(self, cx)
    }
}

impl AsRef<str> for Title<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Signal<'i>(Actor<'i>, SignalType, Actor<'i>, &'i str);

impl<'i> Signal<'i> {
    pub(crate) fn new(l: Actor<'i>, signal: SignalType, r: Actor<'i>, v: &'i str) -> Signal<'i> {
        Signal(l, signal, r, v)
    }

    pub(crate) fn accept<V, C, O>(&self, visitor: &V, cx: &mut C) -> O
    where
        V: Visitor<'i, Context = C, Output = O>,
    {
        visitor.visit_signal(self, cx)
    }

    pub(crate) fn from(&self) -> &Actor<'i> {
        &self.0
    }

    pub(crate) fn to(&self) -> &Actor<'i> {
        &self.2
    }

    pub(crate) fn signal(&self) -> &SignalType {
        &self.1
    }

    pub(crate) fn message(&self) -> &str {
        &self.3
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Participant<'i>(Actor<'i>, Option<Actor<'i>>);

impl<'i> Participant<'i> {
    pub(crate) fn new(actor: Actor<'i>, alias: Option<Actor<'i>>) -> Participant<'i> {
        Participant(actor, alias)
    }

    pub(crate) fn display_name(&self) -> &Option<Actor<'i>> {
        &self.1
    }

    pub(crate) fn actor(&self) -> &Actor<'i> {
        &self.0
    }

    pub(crate) fn accept<V, C, O>(&self, visitor: &V, cx: &mut C) -> O
    where
        V: Visitor<'i, Context = C, Output = O>,
    {
        visitor.visit_participant(self, cx)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Note<'i> {
    LeftOf(Actor<'i>, &'i str),
    RightOf(Actor<'i>, &'i str),
    Over(Actor<'i>, Option<Actor<'i>>, &'i str),
}

impl<'i> Note<'i> {
    pub(crate) fn accept<V, C, O>(&self, visitor: &V, cx: &mut C) -> O
    where
        V: Visitor<'i, Context = C, Output = O>,
    {
        visitor.visit_note(self, cx)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Actor<'i>(&'i str);

impl Actor<'_> {
    pub(crate) fn new(v: &str) -> Actor {
        Actor(v)
    }
}

impl<'i> AsRef<str> for Actor<'i> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SignalType(LineType, ArrowType);

impl From<(LineType, ArrowType)> for SignalType {
    fn from(v: (LineType, ArrowType)) -> SignalType {
        SignalType(v.0, v.1)
    }
}

impl SignalType {
    pub(crate) fn arrow_type(&self) -> ArrowType {
        self.1.clone()
    }

    pub(crate) fn line_type(&self) -> LineType {
        self.0.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LineType {
    Normal,
    Dot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ArrowType {
    None,
    Normal,
    Open,
}
