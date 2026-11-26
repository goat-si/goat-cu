const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { MessageChannel } = require('node:worker_threads');

const source = fs.readFileSync(path.join(__dirname, '../../rust/crates/cua-driver-core/src/browser/tools.rs'), 'utf8');
const match = source.match(/const DOM_EVENT_CLICK_SEQUENCE: &str = "([\s\S]*?)";\n\n\/\/ ── browser_type/);
if (!match) throw new Error('production DOM click sequence not found');
const sequence = JSON.parse('"' + match[1].replace(/\\\n\s*/g, '') + '"');

async function run(removal) {
  const events = [];
  let observer;
  const document = { isConnected: true };
  const element = {
    ownerDocument: document, parentNode: document, isConnected: true,
    getBoundingClientRect: () => ({ left: 0, top: 0, width: 20, height: 20 }),
    getRootNode: () => document,
    contains: value => value === element,
    closest: () => null,
    dispatchEvent(event) {
      events.push(event.type);
      if (removal === 'hover' && event.type === 'pointermove') {
        Promise.resolve().then(() => { element.isConnected = false; });
      }
      if (removal === 'press' && event.type === 'pointerdown') element.isConnected = false;
      if (removal === 'click' && event.type === 'click') {
        element.isConnected = false;
        observer.records.push({ target: document, type: 'childList' });
      }
      if (removal === 'page-change' && event.type === 'click') {
        observer.records.push({ target: document, type: 'childList' });
      }
      return true;
    },
  };
  document.elementFromPoint = () => element;
  class Event { constructor(type) { this.type = type; } }
  class Observer {
    constructor() { this.records = []; observer = this; }
    observe() {}
    takeRecords() { return this.records.splice(0); }
    disconnect() {}
  }
  document.defaultView = { MessageChannel, PointerEvent: Event, MouseEvent: Event, MutationObserver: Observer };
  const result = await (0, eval)(`(${sequence})`).call(element);
  return { events, result };
}

test('a connected click can confirm its page change', async () => {
  const { events, result } = await run('page-change');
  assert.equal(events.filter(event => event === 'click').length, 1);
  assert.equal(result.targetConnected, true);
  assert.equal(result.pageMutations, 1);
});

test('a hover replacement stops before pointerdown', async () => {
  const { events, result } = await run('hover');
  assert.equal(result.targetConnected, false);
  assert.equal(events.includes('pointerdown'), false);
  assert.equal(events.includes('click'), false);
});

test('a detached press target is never clicked', async () => {
  const { events, result } = await run('press');
  assert.equal(result.targetConnected, false);
  assert.equal(events.includes('click'), false);
});

test('a target removed by its click cannot confirm via its removal', async () => {
  const { events, result } = await run('click');
  assert.equal(events.filter(event => event === 'click').length, 1);
  assert.equal(result.targetConnected, false);
});
