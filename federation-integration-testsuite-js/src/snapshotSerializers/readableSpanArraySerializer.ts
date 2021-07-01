import { Plugin } from 'pretty-format';
import { ReadableSpan } from '@opentelemetry/tracing'

export default {
    test(value: any) {
        return value && Array.isArray(value) && value.length > 0 && isReadableSpan(value[0]);
    },

    print(spans: ReadableSpan[]) {
        // This method takes an array of spans and builds up a set of trees containing only the information we are interested in.
        // Spans contain the ID of it's parent, unless it is a root span with no parent.
        //
        // The simplified data model is called a span mirror.
        //
        // The algorithm is:
        // 1. create span mirrors for every span and place it in a map.
        // For each span:
        // 1. find the corresponding mirrors for itself and parent.
        // 2. push the parent in to the parent's children array.

        const root = {children:[]};
        const spanMirrors = new Map<ReadableSpan | undefined, any>();
        spanMirrors.set(undefined, root);
        spans.forEach((s) => spanMirrors.set(s, {name: s.name, attributes:s.attributes, children: [], status: s.status}));
        spans.forEach((s) => {
            const spanMirror = spanMirrors.get(s);
            const parentSpan = spans.find((s2) => s2.spanContext().spanId === s.parentSpanId);
            const parentSpanMirror = spanMirrors.get(parentSpan);
            parentSpanMirror.children.push(spanMirror);
        });

        return JSON.stringify(root.children, undefined, 2);
    },
} as Plugin;

function isReadableSpan(arg: any): arg is ReadableSpan {
    let isSpan = arg && 'kind' in arg && 'startTime' in arg && 'parentSpanId' in arg;
    return isSpan;
}
