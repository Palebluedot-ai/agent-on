#!/usr/bin/env node
// dashboard-check.mjs — 仪表盘 DATA 求值闸（只读，零依赖，Node ≥ 18）。
// 改编自 Dartify #292 的 scripts/check_dashboard_data.mjs（设计与正负例出自那里）。
//
// ── 为什么要有这道闸 ──
// dashboard.html 的 DATA 是 JS 对象字面量。某行末尾漏一个逗号，JS **不报语法错**：
//   ["a","b"]
//   ["c","d","e"]     ⇒ 被读成下标访问 ["a","b"][("c","d","e")] ⇒ undefined
// 两条并成一个值，页面上静默消失；文本闸、结构闸、CI 全绿（Dartify #289：一次漏两处，
// 吞掉两条记录，直到合并前用 node 求值才抓出来）。只有真的求值，才看得见。
//
// ── 查三条（都朝「红」失效）──
// ① 求值后任何层级不许出现 undefined 与数组空洞（`],,` 多一个逗号）。DATA 是纯数据。
// ② 「集合」——从 DATA 出发只经过对象就能到达的数组——里每一条类型一致。
//    条目内部（元组）不查混型：一条里字符串和数字混写是正常的。
// ③ 源码指纹：一个值（`]` `}` `)` 字符串 数字 标识符）后面紧跟 `[` 或 `(`。这是漏逗号在
//    token 层的样子；①② 有盲区（被吞那条恰好取出一个数组、两条的集合被吞成一条），③ 没有，
//    还能报行号。只跳过字符串与注释，不依赖排版。
// fail-closed：找不到或找到多个行首 `const DATA = {`、解析失败、求值失败，一律红。
//
// ── 用法 ──
//   node kit/dashboard-check.mjs [dashboard.html]   # 缺省查当前目录的 dashboard.html
// 退出码：0 通过 / 1 DATA 有问题 / 2 取不出 DATA（文件、标记或语法）
import { readFileSync, realpathSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import vm from 'node:vm';

const MARK = 'const DATA = {';
const MARK_AT_LINE_START = /^const DATA = \{/gm;
const kindOf = (v) => (v === null ? 'null' : Array.isArray(v) ? 'array' : typeof v);
const lineOf = (src, pos) => src.slice(0, pos).split('\n').length;

export function extractData(src) {
  const marks = [...src.matchAll(MARK_AT_LINE_START)];
  if (marks.length !== 1) {
    return { error: marks.length === 0
      ? `找不到行首的「${MARK}」——DATA 改了名或挪了位置,不许静默放行`
      : `行首的「${MARK}」出现了 ${marks.length} 次——不知道该查哪一个` };
  }
  const open = marks[0].index + MARK.length - 1;
  const options = { filename: 'dashboard.html', lineOffset: lineOf(src, open) - 1 };
  // 把其后每一个 `}` 依次当终点试编译:第一个编译得过的就是真终点。
  for (let end = src.indexOf('}', open); end >= 0; end = src.indexOf('}', end + 1)) {
    let script;
    try {
      script = new vm.Script(`(${src.slice(open, end + 1)}\n)`, options);
    } catch {
      continue;
    }
    try {
      return { open, end, data: script.runInNewContext({}, { timeout: 1000 }) };
    } catch (e) {
      return { error: `DATA 求值失败:${e.message}` };
    }
  }
  return { error: 'DATA 解析失败:找不到能闭合它的 `}`(括号不配平或字符串没收尾)' };
}

// ①:undefined 与空洞
function walkValues(v, where, out) {
  if (v === undefined) {
    out.push(`${where} 是 undefined(多半是上一行漏了逗号,两条被读成了下标访问)`);
    return;
  }
  if (Array.isArray(v)) {
    for (let i = 0; i < v.length; i++) {
      if (!(i in v)) out.push(`${where}[${i}] 是数组空洞(多了一个逗号)`);
      else walkValues(v[i], `${where}[${i}]`, out);
    }
  } else if (v && typeof v === 'object') {
    for (const [k, x] of Object.entries(v)) walkValues(x, `${where}.${k}`, out);
  }
}

// ②:只经对象可达的数组 = 集合,条目类型一致
function walkCollections(v, where, out) {
  if (Array.isArray(v)) {
    const kinds = [...new Set(v.filter((_, i) => i in v).map(kindOf))];
    if (kinds.length > 1) out.push(`${where} 这个集合里混了 ${kinds.join(' / ')} 几种条目`);
    return;
  }
  if (v && typeof v === 'object') {
    for (const [k, x] of Object.entries(v)) walkCollections(x, `${where}.${k}`, out);
  }
}

// ③:值后面紧跟 `[` 或 `(`
export function fingerprints(src, from, to) {
  const out = [];
  let prevIsValue = false;
  let i = from;
  while (i < to) {
    const c = src[i];
    if (c === '/' && src[i + 1] === '/') { i = src.indexOf('\n', i); if (i < 0) break; continue; }
    if (c === '/' && src[i + 1] === '*') { const e = src.indexOf('*/', i + 2); i = e < 0 ? to : e + 2; continue; }
    if (/\s/.test(c)) { i++; continue; }
    if (c === '"' || c === "'" || c === '`') {
      let j = i + 1;
      while (j < to && src[j] !== c) j += src[j] === '\\' ? 2 : 1;
      i = j + 1;
      prevIsValue = true;
      continue;
    }
    if (/[0-9]/.test(c) || (c === '.' && /[0-9]/.test(src[i + 1] ?? ''))) {
      while (i < to && /[0-9a-zA-Z_.]/.test(src[i])) i++;
      prevIsValue = true;
      continue;
    }
    if (/[A-Za-z_$]/.test(c)) {
      while (i < to && /[\w$]/.test(src[i])) i++;
      prevIsValue = true;
      continue;
    }
    if ((c === '[' || c === '(') && prevIsValue) {
      out.push(`第 ${lineOf(src, i)} 行:一个值后面紧跟「${c}」——上一行末尾多半漏了逗号`);
    }
    prevIsValue = c === ']' || c === '}' || c === ')';
    i++;
  }
  return out;
}

export function checkSource(src) {
  const got = extractData(src);
  if (got.error) return { code: 2, problems: [got.error] };
  const problems = [];
  walkValues(got.data, 'DATA', problems);
  walkCollections(got.data, 'DATA', problems);
  problems.push(...fingerprints(src, got.open, got.end + 1));
  return { code: problems.length ? 1 : 0, problems };
}

const isMain = (() => {
  try {
    return import.meta.url === pathToFileURL(realpathSync(process.argv[1])).href;
  } catch {
    return false;
  }
})();

if (isMain) {
  const file = process.argv[2] ?? 'dashboard.html';
  let src;
  try {
    src = readFileSync(file, 'utf8');
  } catch (e) {
    console.error(`读不了 ${file}:${e.message}`);
    process.exit(2);
  }
  const { code, problems } = checkSource(src);
  if (code === 0) {
    console.log(`${file}:DATA 求值通过(无 undefined / 空洞 / 混型集合 / 漏逗号指纹)`);
  } else {
    console.error(`${file}:DATA 有 ${problems.length} 处问题`);
    for (const p of problems) console.error(`  - ${p}`);
  }
  process.exit(code);
}
