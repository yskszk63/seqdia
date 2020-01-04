use super::ast::{Document, Note, Participant, Signal, Statement, Title};

pub(crate) trait Visitor<'i> {
    type Context;
    type Output;

    fn default_action(&self, _: &mut Self::Context) -> Self::Output {
        unimplemented!()
    }

    fn visit_document(&self, _document: &Document<'i>, ctx: &mut Self::Context) -> Self::Output {
        self.default_action(ctx)
    }

    fn visit_statement(&self, _statement: &Statement<'i>, ctx: &mut Self::Context) -> Self::Output {
        self.default_action(ctx)
    }

    fn visit_title(&self, _title: &Title<'i>, ctx: &mut Self::Context) -> Self::Output {
        self.default_action(ctx)
    }

    fn visit_signal(&self, _signal: &Signal<'i>, ctx: &mut Self::Context) -> Self::Output {
        self.default_action(ctx)
    }

    fn visit_participant(
        &self,
        _participant: &Participant<'i>,
        ctx: &mut Self::Context,
    ) -> Self::Output {
        self.default_action(ctx)
    }

    fn visit_note(&self, _note: &Note<'i>, ctx: &mut Self::Context) -> Self::Output {
        self.default_action(ctx)
    }
}
