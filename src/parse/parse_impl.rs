#![allow(clippy::upper_case_acronyms)]
use pest::iterators::Pair;
use pest::Parser as _;

use super::ast::{
    Actor, ArrowType, Document, LineType, Note, Participant, Signal, SignalType, Statement, Title,
};

#[derive(pest_derive::Parser)]
#[grammar = "parse/document.pest"]
struct MyParser;

fn parse_signaltype(pair: Pair<Rule>) -> SignalType {
    if pair.as_rule() != Rule::signaltype {
        panic!()
    }
    match pair.as_str() {
        "-" => (LineType::Normal, ArrowType::None).into(),
        "--" => (LineType::Dot, ArrowType::None).into(),
        "->" => (LineType::Normal, ArrowType::Normal).into(),
        "-->" => (LineType::Dot, ArrowType::Normal).into(),
        "->>" => (LineType::Normal, ArrowType::Open).into(),
        "-->>" => (LineType::Dot, ArrowType::Open).into(),
        _ => unreachable!(),
    }
}

fn parse_actor(pair: Pair<Rule>) -> Actor {
    match pair.as_rule() {
        Rule::actor | Rule::actor_quoted => parse_actor(pair.into_inner().next().unwrap()),
        Rule::actor_simple | Rule::actor_quoted_inner => Actor::new(pair.as_str()),
        _ => unreachable!(),
    }
}

fn parse_placement<'i>(pair: Pair<Rule>, actor: Actor<'i>, message: &'i str) -> Note<'i> {
    match pair.as_rule() {
        Rule::placement => parse_placement(pair.into_inner().next().unwrap(), actor, message),
        Rule::leftof => Note::LeftOf(actor, message),
        Rule::rightof => Note::RightOf(actor, message),
        _ => unreachable!(),
    }
}

fn parse_statement(pair: Pair<Rule>) -> Statement {
    match pair.as_rule() {
        Rule::title => {
            let title = Title::new(pair.into_inner().as_str());
            Statement::Title(title)
        }

        Rule::signal => {
            let mut inner = pair.into_inner();
            let left = parse_actor(inner.next().unwrap());
            let signaltype = parse_signaltype(inner.next().unwrap());
            let right = parse_actor(inner.next().unwrap());
            let message = inner.next().unwrap().as_str();
            let signal = Signal::new(left, signaltype, right, message);
            Statement::Signal(signal)
        }

        Rule::participant => {
            let mut inner = pair.into_inner();
            let actor = parse_actor(inner.next().unwrap());
            let participant = if let Some(pair) = inner.next() {
                Participant::new(actor, Some(parse_actor(pair)))
            } else {
                Participant::new(actor, None)
            };
            Statement::Participant(participant)
        }
        Rule::note => {
            let mut inner = pair.into_inner();
            let maybe_placement = inner.next().unwrap();
            let note = match maybe_placement.as_rule() {
                Rule::placement => {
                    let actor = parse_actor(inner.next().unwrap());
                    let message = inner.next().unwrap().as_str();
                    parse_placement(maybe_placement, actor, message)
                }
                Rule::over => {
                    let actor = parse_actor(inner.next().unwrap());
                    let maybe_actor = inner.next().unwrap();
                    if maybe_actor.as_rule() == Rule::actor {
                        Note::Over(
                            actor,
                            Some(parse_actor(maybe_actor)),
                            inner.next().unwrap().as_str(),
                        )
                    } else {
                        Note::Over(actor, None, maybe_actor.as_str())
                    }
                }
                _ => unreachable!(),
            };
            Statement::Note(note)
        }
        e => unreachable!("{:?}", e),
    }
}

fn parse_document(pair: Pair<Rule>) -> Vec<Statement> {
    match pair.as_rule() {
        Rule::document => {
            let mut result = vec![];
            for inner in pair.into_inner() {
                result.extend(parse_document(inner))
            }
            result
        }
        Rule::statement => {
            let mut result = vec![];
            for inner in pair.into_inner() {
                let statement = parse_statement(inner);
                result.push(statement)
            }
            result
        }
        Rule::EOI => vec![],
        e => unreachable!("x {:?}", e),
    }
}

pub(crate) fn parse(input: &str) -> Result<Document, pest::error::Error<Rule>> {
    let r = MyParser::parse(Rule::document, input)?;
    let mut result = vec![];
    for pair in r {
        result.extend(parse_document(pair));
    }
    Ok(result.into())
}
