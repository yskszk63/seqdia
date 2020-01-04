import * as wasm from "seqdia";
import * as cm from "codemirror";
import "codemirror/lib/codemirror.css"

const editor = cm.fromTextArea(document.querySelector('textarea'), {
    lineNumbers: true,
    lineWrapping: true,
});

const widgets = [];
function updateAnnotations(infos) {
    editor.operation(() => {
        while (widgets.length) {
            let widget = widgets.shift();
            editor.removeLineWidget(widget);
        }

        infos.forEach(info => {
            const message = info.message || String(info);
            const line = info.line || 0;
            const msg = Object.assign(document.createElement('div'), { });
            Object.assign(msg.appendChild(document.createElement('pre')), {
                textContent: message,
                className: 'error-msg',
            });
            widgets.push(editor.addLineWidget(line - 1, msg, {coverGutter: true, noHScroll: true}));
        });
    });
}

if (location.hash.length > 1) {
    const {text, svg} = wasm.load_and_gen(location.hash);
    editor.setValue(text);
    document.querySelector('output').innerHTML = svg;
}

editor.on('change', (cm, ch) => {
    const text = cm.getValue();
    document.querySelector('body').classList.add('incomplete');
    let result;
    try {
        result = wasm.pickle_and_gen(text);
    } catch (e) {
        updateAnnotations([e]);
        throw e;
    }
    document.querySelector('body').classList.remove('incomplete');
    const {pickled, svg} = result;
    updateAnnotations([]);
    location.hash = pickled;
    document.querySelector('output').innerHTML = svg;
});
