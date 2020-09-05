mod paper;
mod parse;

use std::fmt;
use std::hash::Hasher as _;
use std::rc::Rc;
use std::sync::Mutex;

use lz4_compression::prelude::{compress, decompress};
use wasm_bindgen::prelude::*;
use unicode_width::UnicodeWidthStr;
use pest::error::LineColLocation;
use js_sys::{Object, JsString, Reflect};
use web_sys::Element;
use thiserror::Error;

use paper::{MarkerEnd, Paper, Path, Rect, Text, TextAnchor};
use parse::{
    Actor, ArrowType, Document, LineType, Note, Participant, Signal, Statement, Title, Visitor,
};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const FONT_SIZE: isize = 16; // FIXME

const DIAGRAM_MARGIN: isize = 10;

const ACTOR_MARGIN: isize = 10;
const ACTOR_PADDING: isize = 10;

const SIGNAL_MARGIN: isize = 10;
const SIGNAL_PADDING: isize = 10;

/*
const NOTE_MARGIN: isize = 10;
const NOTE_PADDING: isize = 5;
const NOTE_OVERLAP: isize = 15;
*/

const TITLE_MARGIN: isize = 0;
const TITLE_PADDING: isize = 5;

const SELF_SIGNAL_WIDTH: isize = 20;

fn text_bbox(text: &str) -> Rectangle {
    let width = UnicodeWidthStr::width(text) as isize * (FONT_SIZE / 2) + ((text.len() / 5 * 6) as isize);
    Rectangle::new(0, 0, width, FONT_SIZE)
}

#[derive(Debug, Error, Clone)]
struct ParseError {
    line: Option<usize>,
    message: String,
}

impl From<pest::error::Error<parse::Rule>> for ParseError {
    fn from(v: pest::error::Error<parse::Rule>) -> Self {
        let line = match v.line_col {
            LineColLocation::Pos((line, _)) => line,
            LineColLocation::Span((line, _), _) => line,
        };
        Self {
            line: Some(line),
            message: v.to_string(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

#[derive(Debug)]
struct LayoutTitle;

impl<'i> Visitor<'i> for LayoutTitle {
    type Output = ();
    type Context = Layout<'i>;

    fn visit_document(&self, document: &Document<'i>, ctx: &mut Self::Context) -> Self::Output {
        for statement in document {
            statement.accept(self, ctx)
        }
    }

    fn visit_statement(&self, statement: &Statement<'i>, ctx: &mut Self::Context) -> Self::Output {
        match statement {
            Statement::Title(title) => title.accept(self, ctx),
            Statement::Signal(signal) => signal.accept(self, ctx),
            Statement::Participant(participant) => participant.accept(self, ctx),
            Statement::Note(note) => note.accept(self, ctx),
        }
    }

    fn visit_title(&self, title: &Title<'i>, ctx: &mut Self::Context) -> Self::Output {
        if ctx.title.is_none() {
            let bbox = text_bbox(&title.as_ref());
            ctx.title = Some((
                title.clone(),
                Rectangle::new(
                    DIAGRAM_MARGIN,
                    DIAGRAM_MARGIN,
                    bbox.w + (TITLE_PADDING + TITLE_MARGIN) * 2,
                    bbox.h + (TITLE_PADDING + TITLE_MARGIN) * 2,
                ),
            ))
        }
    }

    fn visit_signal(&self, _signal: &Signal<'i>, _ctx: &mut Self::Context) -> Self::Output {}

    fn visit_participant(
        &self,
        _participant: &Participant<'i>,
        _ctx: &mut Self::Context,
    ) -> Self::Output {
    }

    fn visit_note(&self, _note: &Note<'i>, _ctx: &mut Self::Context) -> Self::Output {}
}

#[derive(Debug)]
struct LayoutCalculator;

impl<'i> Visitor<'i> for LayoutCalculator {
    type Output = ();
    type Context = Layout<'i>;

    fn visit_document(&self, document: &Document<'i>, ctx: &mut Self::Context) -> Self::Output {
        for statement in document {
            statement.accept(self, ctx)
        }
    }

    fn visit_statement(&self, statement: &Statement<'i>, ctx: &mut Self::Context) -> Self::Output {
        match statement {
            Statement::Title(title) => title.accept(self, ctx),
            Statement::Signal(signal) => signal.accept(self, ctx),
            Statement::Participant(participant) => participant.accept(self, ctx),
            Statement::Note(note) => note.accept(self, ctx),
        }
    }

    fn visit_title(&self, _title: &Title<'i>, _ctx: &mut Self::Context) -> Self::Output {}

    fn visit_signal(&self, signal: &Signal<'i>, ctx: &mut Self::Context) -> Self::Output {
        let mut add_actor = |actor: &Actor<'i>| {
            if ctx.pos_by_actor(actor).is_none() {
                let x = ctx.actors.iter().map(|(_, _, r)| r.w).sum::<isize>();
                let bbox = text_bbox(actor.as_ref());
                ctx.actors.push((
                    actor.clone(),
                    actor.clone(),
                    Rectangle::new(
                        bbox.x + x,
                        bbox.y,
                        bbox.w + (ACTOR_MARGIN + ACTOR_PADDING) * 2,
                        bbox.h + (ACTOR_MARGIN + ACTOR_PADDING) * 2,
                    ),
                ))
            }
        };

        add_actor(signal.from());
        add_actor(signal.to());

        let bbox = text_bbox(signal.message());
        let y = ctx.signals.iter().map(|(_, r)| r.h).sum::<isize>();
        let x1 = ctx.pos_by_actor(signal.from()).unwrap().center_x();
        let x2 = ctx.pos_by_actor(signal.to()).unwrap().center_x();
        ctx.signals.push((
            SignalKind::Signal(signal.clone()),
            Rectangle::new(x1, y, x2, bbox.h + (SIGNAL_MARGIN + SIGNAL_PADDING) * 2),
        ));
    }

    fn visit_participant(
        &self,
        participant: &Participant<'i>,
        ctx: &mut Self::Context,
    ) -> Self::Output {
        let mut add_actor = |actor: &Actor<'i>, display_name: &Option<Actor<'i>>| {
            if ctx.pos_by_actor(actor).is_none() {
                let x = ctx.actors.iter().map(|(_, _, r)| r.w).sum::<isize>();
                let display_name = display_name.as_ref().unwrap_or_else(|| actor).clone();
                let bbox = text_bbox(display_name.as_ref());
                ctx.actors.push((
                    actor.clone(),
                    display_name,
                    Rectangle::new(
                        bbox.x + x,
                        bbox.y,
                        bbox.w + (ACTOR_MARGIN + ACTOR_PADDING) * 2,
                        bbox.h + (ACTOR_MARGIN + ACTOR_PADDING) * 2,
                    ),
                ))
            }
        };

        add_actor(participant.actor(), participant.display_name());
    }

    fn visit_note(&self, _note: &Note<'i>, _ctx: &mut Self::Context) -> Self::Output {}
}

#[derive(Debug, Clone)]
struct Rectangle {
    x: isize,
    y: isize,
    w: isize,
    h: isize,
}

impl Rectangle {
    fn new(x: isize, y: isize, w: isize, h: isize) -> Self {
        Self { x, y, w, h }
    }

    fn center_x(&self) -> isize {
        self.x + self.w / 2
    }
}

#[derive(Debug)]
enum SignalKind<'i> {
    Signal(Signal<'i>),
    //Note(Note<'i>),
}

#[derive(Debug, Default)]
struct Layout<'i> {
    title: Option<(Title<'i>, Rectangle)>,
    actors: Vec<(Actor<'i>, Actor<'i>, Rectangle)>,
    signals: Vec<(SignalKind<'i>, Rectangle)>,
    width: isize,
    height: isize,
}

impl<'i> Layout<'i> {
    fn pos_by_actor(&self, target: &Actor<'_>) -> Option<Rectangle> {
        for (alias, _, rectangle) in &self.actors {
            if alias == target {
                return Some(rectangle.clone());
            }
        }
        None
    }
}

#[derive(Default)]
struct Wobble {
    hasher: fnv::FnvHasher,
}

impl Wobble {
    fn wobble(&mut self, x1: isize, y1: isize, x2: isize, y2: isize) -> String {
        let factor = (((x2 - x1).pow(2) + (y2 - y1).pow(2)) as f32).sqrt() / 25.0;
        self.hasher.write_u32(factor.to_bits());

        let r1 = (((self.hasher.finish() % 60) as f32) / 100.0) + 0.2;
        self.hasher.write_u32(r1.to_bits());
        let r2 = (((self.hasher.finish() % 60) as f32) / 100.0) + 0.2;
        self.hasher.write_u32(r2.to_bits());

        let xfactor = if self.hasher.finish() % 2 == 0 {factor} else {-factor};
        self.hasher.write_u32(xfactor.to_bits());
        let yfactor = if self.hasher.finish() % 2 == 0 {factor} else {-factor};
        self.hasher.write_u32(yfactor.to_bits());

        let p1x = ((x2 - x1) as f32) * r1 + (x1 as f32) + xfactor;
        let p1y = ((y2 - y1) as f32) * r1 + (y1 as f32) + yfactor;

        let p2x = ((x2 - x1) as f32) * r2 + (x1 as f32) - xfactor;
        let p2y = ((y2 - y1) as f32) * r2 + (y1 as f32) - yfactor;

        format!("C{:.1},{:.1} {:.1},{:.1} {},{}", p1x, p1y, p2x, p2y, x2, y2)
    }
}

#[derive(Debug)]
struct SequenceDiagram<'i> {
    document: Document<'i>,
}

impl<'i> SequenceDiagram<'i> {
    fn parse(text: &'i str) -> Result<Self, ParseError> {
        let document = parse::parse(text)?;
        Ok(Self { document })
    }

    fn draw(&self) -> String {
        let layout = self.layout();

        let title_height =
            layout.title.as_ref().map(|(_, r)| r.h).unwrap_or_else(|| 0) + DIAGRAM_MARGIN;
        let signal_height = layout.signals.iter().map(|(_, r)| r.h).sum::<isize>();
        let actor_height = layout
            .actors
            .iter()
            .map(|(_, _, r)| r.h)
            .max()
            .unwrap_or_else(|| 0);

        let mut paper = Paper::builder()
            .w(layout
                .actors
                .iter()
                .map(|(_, _, r)| r.x + r.w)
                .max()
                .unwrap_or_else(|| 0))
            .h(title_height + signal_height + (actor_height * 2))
            .build();
        let mut w = Wobble::default();

        self.draw_title(&mut paper, &layout, &mut w);
        self.draw_actor(&mut paper, &layout, &mut w);
        self.draw_signals(&mut paper, &layout, &mut w);

        paper.to_svg_string()
    }

    fn layout(&self) -> Layout {
        let mut layout = Layout::default();

        self.document.accept(&mut LayoutTitle, &mut layout);
        self.document.accept(&mut LayoutCalculator, &mut layout);

        layout
    }

    fn draw_title(&self, paper: &mut Paper, layout: &Layout, w: &mut Wobble) {
        if let Some((title, rectangle)) = &layout.title {
            self.draw_text_box(
                paper,
                rectangle,
                title.as_ref(),
                TITLE_MARGIN,
                TITLE_PADDING,
                w,
            );
        }
    }

    fn draw_actor(&self, paper: &mut Paper, layout: &Layout, w: &mut Wobble) {
        let y = layout.title.as_ref().map(|(_, r)| r.h).unwrap_or_else(|| 0) + DIAGRAM_MARGIN;
        let signal_height = layout.signals.iter().map(|(_, r)| r.h).sum::<isize>();

        for (_, actor, rectangle) in &layout.actors {
            let mut rectangle = rectangle.clone();
            rectangle.y = y;
            self.draw_text_box(
                paper,
                &rectangle,
                &actor.as_ref(),
                ACTOR_MARGIN,
                ACTOR_PADDING,
                w,
            );

            rectangle.y = y + rectangle.h + signal_height;
            self.draw_text_box(
                paper,
                &rectangle,
                &actor.as_ref(),
                ACTOR_MARGIN,
                ACTOR_PADDING,
                w,
            );

            self.draw_line(
                paper,
                rectangle.center_x(),
                y + rectangle.h - ACTOR_MARGIN,
                rectangle.center_x(),
                y + rectangle.h + ACTOR_MARGIN + signal_height,
                None,
                false,
                w,
            );
        }
    }

    fn draw_signals(&self, paper: &mut Paper, layout: &Layout, w: &mut Wobble) {
        let y = layout.title.as_ref().map(|(_, r)| r.h).unwrap_or_else(|| 0) + DIAGRAM_MARGIN;
        let y2 = layout
            .actors
            .iter()
            .map(|(_, _, r)| r.h)
            .max()
            .unwrap_or_else(|| 0);

        for (signal, rectangle) in &layout.signals {
            match signal {
                SignalKind::Signal(signal) => {
                    if signal.from() == signal.to() {
                        self.draw_text(
                            paper,
                            signal.message(),
                            rectangle.x + SELF_SIGNAL_WIDTH,
                            rectangle.y + y + y2,
                            SIGNAL_MARGIN,
                            SIGNAL_PADDING,
                            true,
                        );

                        self.draw_line(
                            paper,
                            rectangle.x,
                            rectangle.y + y + y2 + SIGNAL_MARGIN,
                            rectangle.x + SELF_SIGNAL_WIDTH,
                            rectangle.y + y + y2 + SIGNAL_MARGIN,
                            None,
                            signal.signal().line_type() == LineType::Dot,
                            w,
                        );
                        self.draw_line(
                            paper,
                            rectangle.x + SELF_SIGNAL_WIDTH,
                            rectangle.y + y + y2 + SIGNAL_MARGIN,
                            rectangle.x + SELF_SIGNAL_WIDTH,
                            rectangle.y + y + y2 + rectangle.h,
                            None,
                            signal.signal().line_type() == LineType::Dot,
                            w,
                        );
                        self.draw_line(
                            paper,
                            rectangle.x + SELF_SIGNAL_WIDTH,
                            rectangle.y + y + y2 + rectangle.h,
                            rectangle.x,
                            rectangle.y + y + y2 + rectangle.h,
                            match signal.signal().arrow_type() {
                                ArrowType::Normal => Some(MarkerEnd::ArrowBlock),
                                ArrowType::Open => Some(MarkerEnd::ArrowOpen),
                                ArrowType::None => None,
                            },
                            signal.signal().line_type() == LineType::Dot,
                            w,
                        );
                    } else {
                        self.draw_text(
                            paper,
                            signal.message(),
                            rectangle.x,
                            rectangle.y + y + y2,
                            SIGNAL_MARGIN,
                            SIGNAL_PADDING,
                            rectangle.x < rectangle.w,
                        );

                        self.draw_line(
                            paper,
                            rectangle.x,
                            rectangle.y + y + y2 + rectangle.h,
                            rectangle.w,
                            rectangle.y + y + y2 + rectangle.h,
                            match signal.signal().arrow_type() {
                                ArrowType::Normal => Some(MarkerEnd::ArrowBlock),
                                ArrowType::Open => Some(MarkerEnd::ArrowOpen),
                                ArrowType::None => None,
                            },
                            signal.signal().line_type() == LineType::Dot,
                            w,
                        );
                    }
                }
                //_ => {}
            }
        }
    }

    fn draw_text(
        &self,
        paper: &mut Paper,
        text: &str,
        x: isize,
        y: isize,
        margin: isize,
        padding: isize,
        anchor_left: bool,
    ) {
        let x = if anchor_left {
            x + margin + padding
        } else {
            x - (margin + padding)
        };
        let y = y + margin + padding;
        let anchor = if anchor_left {
            TextAnchor::Start
        } else {
            TextAnchor::End
        };
        let bbox = text_bbox(text);

        paper.push(
            Rect::new(
                if anchor_left { x } else { x - bbox.w },
                y,
                bbox.w,
                bbox.h + margin + padding,
            )
            .with_stroke("none")
            .with_stroke_width(0)
            .with_fill("white")
            .with_fill_opacity(70),
        );
        paper.push(Text::new(x, y, text.to_string()).with_text_anchor(anchor));
    }

    fn draw_text_box(
        &self,
        paper: &mut Paper,
        rectangle: &Rectangle,
        text: &str,
        margin: isize,
        padding: isize,
        ww: &mut Wobble,
    ) {
        let x = rectangle.x + margin;
        let y = rectangle.y + margin;
        let w = rectangle.w - 2 * margin;
        let h = rectangle.h - 2 * margin;

        //paper.push(Rect::new(x, y, w, h));
        paper.push(Path::new(format!("M{},{}{}{}{}{}",
                    x, y,
                    ww.wobble(x, y, x + w, y),
                    ww.wobble(x + w, y, x + w, y + h),
                    ww.wobble(x + w, y + h, x, y + h),
                    ww.wobble(x, y + h, x, y),
                    )));
        paper.push(Text::new(x + padding, y + padding, text.to_string()));
    }

    fn draw_line(
        &self,
        paper: &mut Paper,
        x1: isize,
        y1: isize,
        x2: isize,
        y2: isize,
        marker_end: Option<MarkerEnd>,
        dash: bool,
        w: &mut Wobble,
    ) {
        //let mut path = Path::new(format!("M{},{} L{},{}", x1, y1, x2, y2));
        let mut path = Path::new(format!("M{},{}{}", x1, y1, w.wobble(x1, y1, x2, y2)));
        if let Some(marker_end) = marker_end {
            path = path.with_marker_end(marker_end);
        }
        if dash {
            path = path.with_stroke_dasharray("6px,2px");
        }
        paper.push(path);
    }
}

fn pickle_and_gen(text: &str) -> Result<(String, String), ParseError> {
    let compressed = compress(text.as_bytes());
    let pickled = base64::encode_config(
        &compressed,
        base64::Config::new(base64::CharacterSet::UrlSafe, false),
    );
    let pickled = format!("/v1/{}", pickled);
    let svg = generate(text)?;

    Ok((pickled, svg))
}

#[derive(Error, Debug)]
enum LoadAndGenError {
    #[error("unexpected hash")]
    UnexpectedHash,

    #[error("decode error")]
    DecodeError(#[from] base64::DecodeError),

    #[error("decompress error")]
    DecompressError(lz4_compression::decompress::Error),

    #[error("from utf8 error")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    #[error("parse error")]
    ParseError(#[from] ParseError),
}

fn load_and_gen(hash: &str) -> Result<(String, String), LoadAndGenError> {
    if hash.is_empty() {
        return Ok(("".to_owned(), "".to_owned()));
    }
    let mut parts = hash.split('/');
    parts.next();
    match parts.next() {
        Some("v1") => {}
        _ => return Err(LoadAndGenError::UnexpectedHash),
    }
    let pickled = if let Some(text) = parts.next() {
        text
    } else {
        return Err(LoadAndGenError::UnexpectedHash);
    };
    let compressed = base64::decode_config(
        &pickled,
        base64::Config::new(base64::CharacterSet::UrlSafe, false),
    )?;
    let text = decompress(&compressed).map_err(LoadAndGenError::DecompressError)?;
    let text = String::from_utf8(text)?;

    let svg = generate(&text)?;

    Ok((text, svg))
}

fn generate(text: &str) -> Result<String, ParseError> {
    let result = SequenceDiagram::parse(text)?;
    Ok(result.draw())
}

#[wasm_bindgen(module = "codemirror")]
extern "C" {
    type CodeMirror;

    fn fromTextArea(element: &web_sys::Element, options: &JsValue) -> CodeMirror;

    #[wasm_bindgen(method)]
    fn on(this: &CodeMirror, when: &JsValue, f: &JsValue);

    #[wasm_bindgen(method)]
    fn getValue(this: &CodeMirror) -> String;

    #[wasm_bindgen(method)]
    fn setValue(this: &CodeMirror, value: &str);

    #[wasm_bindgen(method)]
    fn operation(this: &CodeMirror, f: &dyn Fn());

    #[wasm_bindgen(method)]
    fn addLineWidget(this: &CodeMirror, line: usize, e: &Element, options: &JsValue) -> Element;

    #[wasm_bindgen(method)]
    fn removeLineWidget(this: &CodeMirror, e: &Element);
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    wasm_logger::init(Default::default());

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    let output = document.query_selector("output").unwrap().unwrap();

    let editor = unsafe {
        let options = Object::new();
        Reflect::set(&options, &JsString::from("lineNumbers"), &JsValue::TRUE).unwrap();
        Reflect::set(&options, &JsString::from("lineWrapping"), &JsValue::TRUE).unwrap();
        fromTextArea(&document.query_selector("textarea").unwrap().unwrap(), &options)
    };
    let editor = Rc::new(editor);

    let editor2 = editor.clone();
    let widgets: Rc<Mutex<Vec<Element>>> = Rc::new(Mutex::new(vec![]));
    let document = Rc::new(document);
    let update_annotations = move |infos: Vec<ParseError>| {
        let editor = editor2.clone();
        let widgets = widgets.clone();
        let document = document.clone();
        editor2.operation(&move || {
            let mut widgets = widgets.lock().unwrap();
            for widget in widgets.iter() {
                editor.removeLineWidget(&widget);
            }
            widgets.clear();

            for info in &infos {
                let message = &info.message;
                let line = info.line.unwrap_or(0);

                let pre = document.create_element("pre").unwrap();
                pre.set_text_content(Some(message));
                let msg = document.create_element("div").unwrap();
                msg.class_list().add_1("error-msg").unwrap();
                msg.append_child(&pre).unwrap();

                let opt = Object::new();
                unsafe {
                    Reflect::set(&opt, &JsString::from("coverGutter"), &JsValue::TRUE).unwrap();
                    Reflect::set(&opt, &JsString::from("noHScroll"), &JsValue::TRUE).unwrap();
                };
                let widget = editor.addLineWidget(line - 1, &msg, &opt);
                widgets.push(widget);
            }
        });
    };

    let hash = window.location().hash().unwrap();
    if hash.len() > 1 {
        let (text, svg) = load_and_gen(&hash).unwrap();
        editor.setValue(&text);
        output.set_inner_html(&svg);
    }

    let c = Closure::wrap(Box::new(move |cm: CodeMirror, _| {
        let text = cm.getValue();
        body.class_list().add_1("incomplete").unwrap();
        let (pickled, svg) = match pickle_and_gen(&text) {
            Ok((pickled, svg)) => (pickled, svg),
            Err(e) => {
                log::error!("{:?}", e);
                update_annotations(vec![e]);
                return;
            }
        };
        body.class_list().remove_1("incomplete").unwrap();
        update_annotations(vec![]);
        window.location().set_hash(&pickled).unwrap();
        output.set_inner_html(&svg);
    }) as Box<dyn Fn(CodeMirror, JsValue)>);

    editor.on(&JsString::from("change"), c.as_ref());
    c.forget();

    Ok(())
}

/*
fn main() {
    let input = r###"title titleok
participant aaa as "A B C"
aaa -> bbb: ok
aaa ->> bbb: ok
aaa - bbb: ok
aaa --> bbb: ok
aaa -->> bbb: ok
aaa -->> aaa: self
aaa -> aaa: self
note left of aaa: hello
note right of aaa: hello2
note over aaa: hello3
note over aaa, bbb: hello4
" aaa" -> bbb: ok2"###;

    let result = SequenceDiagram::parse(input).unwrap();
    let output = result.draw();
    println!("{}", output);
}
*/
