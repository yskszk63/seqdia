use xmlwriter::XmlWriter;

#[derive(Debug)]
pub(crate) struct PaperBuilder {
    w: isize,
    h: isize,
}

impl Default for PaperBuilder {
    fn default() -> Self {
        Self { w: 512, h: 342 }
    }
}

impl PaperBuilder {
    pub(crate) fn w(&mut self, w: isize) -> &mut Self {
        self.w = w;
        self
    }

    pub(crate) fn h(&mut self, h: isize) -> &mut Self {
        self.h = h;
        self
    }

    pub(crate) fn build(&mut self) -> Paper {
        Paper {
            w: self.w,
            h: self.h,
            elements: vec![],
        }
    }
}

#[derive(Debug)]
pub(crate) enum TextAnchor {
    Start,
    #[allow(dead_code)]
    Middle,
    End,
}

#[derive(Debug)]
pub(crate) struct Text {
    x: isize,
    y: isize,
    text: String,
    text_anchor: Option<TextAnchor>,
}

impl From<Text> for Element {
    fn from(v: Text) -> Self {
        Self::Text(v)
    }
}

impl Text {
    pub(crate) fn new(x: isize, y: isize, text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            x,
            y,
            text,
            text_anchor: None,
        }
    }

    pub(crate) fn with_text_anchor(self, text_anchor: TextAnchor) -> Self {
        let text_anchor = Some(text_anchor);
        Self {
            text_anchor,
            ..self
        }
    }

    fn write_svg(&self, writer: &mut XmlWriter) {
        writer.start_element("text");
        writer.write_attribute_fmt("x", format_args!("{}", self.x));
        writer.write_attribute_fmt("y", format_args!("{}", self.y));
        writer.write_attribute(
            "text-anchor",
            match self.text_anchor.as_ref().unwrap_or(&TextAnchor::Start) {
                TextAnchor::Start => "start",
                TextAnchor::Middle => "middle",
                TextAnchor::End => "end",
            },
        );
        writer.write_attribute("font-size", "16px");

        for line in self.text.lines() {
            writer.start_element("tspan");
            writer.write_attribute_fmt("dy", format_args!("{}", 16.0 * 1.2)); // fontSize * leading
            writer.write_attribute_fmt("x", format_args!("{}", self.x));
            writer.write_text(line);
            writer.end_element();
        }

        writer.end_element();
    }
}

#[derive(Debug)]
pub(crate) enum MarkerEnd {
    ArrowBlock,
    ArrowOpen,
}

#[derive(Debug)]
pub(crate) struct Path {
    d: String,
    marker_end: Option<MarkerEnd>,
    stroke_dasharray: Option<String>,
}

impl From<Path> for Element {
    fn from(v: Path) -> Self {
        Self::Path(v)
    }
}

impl Path {
    pub(crate) fn new(d: impl Into<String>) -> Self {
        let d = d.into();
        Self {
            d,
            marker_end: None,
            stroke_dasharray: None,
        }
    }

    pub(crate) fn with_marker_end(self, marker_end: MarkerEnd) -> Self {
        let marker_end = Some(marker_end);
        Self { marker_end, ..self }
    }

    pub(crate) fn with_stroke_dasharray(self, stroke_dasharray: impl Into<String>) -> Self {
        let stroke_dasharray = Some(stroke_dasharray.into());
        Self {
            stroke_dasharray,
            ..self
        }
    }

    fn write_svg(&self, writer: &mut XmlWriter) {
        writer.start_element("path");
        writer.write_attribute("fill", "none");
        writer.write_attribute("stroke", "#000");
        writer.write_attribute("stroke-width", "2px");
        writer.write_attribute("d", &self.d);
        if let Some(marker_end) = &self.marker_end {
            match marker_end {
                MarkerEnd::ArrowBlock => writer.write_attribute("marker-end", "url(#arrowblock)"),
                MarkerEnd::ArrowOpen => writer.write_attribute("marker-end", "url(#arrowopen)"),
            }
        }
        if let Some(stroke_dasharray) = &self.stroke_dasharray {
            writer.write_attribute("stroke-dasharray", stroke_dasharray)
        }
        writer.end_element();
    }
}

#[derive(Debug)]
pub(crate) struct Rect {
    x: isize,
    y: isize,
    width: isize,
    height: isize,
    r: Option<usize>,
    stroke: Option<String>,
    stroke_width: Option<usize>,
    fill: Option<String>,
    fill_opacity: Option<usize>,
}

impl From<Rect> for Element {
    fn from(v: Rect) -> Self {
        Self::Rect(v)
    }
}

impl Rect {
    pub(crate) fn new(x: isize, y: isize, width: isize, height: isize) -> Self {
        Self {
            x,
            y,
            width,
            height,
            r: None,
            stroke: None,
            stroke_width: None,
            fill: None,
            fill_opacity: None,
        }
    }

    /*
    pub(crate) fn with_r(self, r: usize) -> Self {
        let r = Some(r);
        Self { r, ..self }
    }
    */

    pub(crate) fn with_stroke(self, stroke: impl Into<String>) -> Self {
        let stroke = Some(stroke.into());
        Self { stroke, ..self }
    }

    pub(crate) fn with_stroke_width(self, stroke_width: usize) -> Self {
        let stroke_width = Some(stroke_width);
        Self {
            stroke_width,
            ..self
        }
    }

    pub(crate) fn with_fill(self, fill: impl Into<String>) -> Self {
        let fill = Some(fill.into());
        Self { fill, ..self }
    }

    pub(crate) fn with_fill_opacity(self, fill_opacity: usize) -> Self {
        let fill_opacity = Some(fill_opacity);
        Self {
            fill_opacity,
            ..self
        }
    }

    fn write_svg(&self, writer: &mut XmlWriter) {
        writer.start_element("rect");
        writer.write_attribute_fmt("x", format_args!("{}", self.x));
        writer.write_attribute_fmt("y", format_args!("{}", self.y));
        writer.write_attribute_fmt("width", format_args!("{}", self.width));
        writer.write_attribute_fmt("height", format_args!("{}", self.height));
        writer.write_attribute_fmt("rx", format_args!("{}", self.r.unwrap_or(0)));
        writer.write_attribute_fmt("ry", format_args!("{}", self.r.unwrap_or(0)));
        writer.write_attribute_fmt(
            "fill",
            format_args!(
                "{}",
                self.fill.clone().unwrap_or_else(|| "none".to_string())
            ),
        );
        if let Some(fill_opacity) = self.fill_opacity {
            writer.write_attribute_fmt("fill-opacity", format_args!("{}%", fill_opacity));
        }
        writer.write_attribute_fmt(
            "stroke",
            format_args!(
                "{}",
                self.fill.clone().unwrap_or_else(|| "#000".to_string())
            ),
        );
        writer.write_attribute_fmt(
            "stroke-width",
            format_args!("{}px", self.stroke_width.unwrap_or(2)),
        );
        writer.end_element();
    }
}

#[derive(Debug)]
pub(crate) enum Element {
    Text(Text),
    Path(Path),
    Rect(Rect),
}

impl Element {
    fn write_svg(&self, writer: &mut XmlWriter) {
        match self {
            Self::Text(e) => e.write_svg(writer),
            Self::Path(e) => e.write_svg(writer),
            Self::Rect(e) => e.write_svg(writer),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Paper {
    w: isize,
    h: isize,
    elements: Vec<Element>,
}

impl Paper {
    pub(crate) fn builder() -> PaperBuilder {
        PaperBuilder::default()
    }

    pub(crate) fn to_svg_string(&self) -> String {
        let mut writer = XmlWriter::new(Default::default());

        writer.start_element("svg");
        writer.write_attribute("xmlns", "http://www.w3.org/2000/svg");
        writer.write_attribute_fmt("width", format_args!("{}", self.w));
        writer.write_attribute_fmt("height", format_args!("{}", self.h));
        writer.write_attribute_fmt("version", format_args!("{}", 1.1));

        writer.start_element("desc");
        writer.end_element();

        writer.start_element("defs");

        writer.start_element("marker");
        writer.write_attribute("id", "arrowblock");
        writer.write_attribute("viewBox", "0 0 5 5");
        writer.write_attribute("markerWidth", "5");
        writer.write_attribute("markerHeight", "5");
        writer.write_attribute("orient", "auto");
        writer.write_attribute("refX", "5");
        writer.write_attribute("refY", "2.5");
        writer.start_element("path");
        writer.write_attribute("d", "M 0 0 L 5 2.5 L 0 5 z");
        writer.end_element();
        writer.end_element();

        writer.start_element("marker");
        writer.write_attribute("id", "arrowopen");
        writer.write_attribute("viewBox", "0 0 9.6 16");
        writer.write_attribute("markerWidth", "4");
        writer.write_attribute("markerHeight", "16");
        writer.write_attribute("orient", "auto");
        writer.write_attribute("refX", "9.6");
        writer.write_attribute("refY", "8");
        writer.start_element("path");
        writer.write_attribute("d", "M 9.6,8 1.92,16 0,13.7 5.76,8 0,2.286 1.92,0 9.6,8 z");
        writer.end_element();
        writer.end_element();
        writer.end_element();

        for element in &self.elements {
            element.write_svg(&mut writer);
        }

        writer.end_document()
    }

    pub(crate) fn push(&mut self, element: impl Into<Element>) {
        self.elements.push(element.into());
    }
}
