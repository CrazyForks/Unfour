// Mark Tailwind v4 internal variables as non-token metadata before
// design-sync uploads the bundle. These are framework state, not design tokens.
import { existsSync, readFileSync, writeFileSync } from 'node:fs';

const KIND_OTHER = '/* @kind other */';
const CSS_FILES = ['ds-bundle/_ds_bundle.css', 'ds-bundle/styles.css'];
const README_FILE = 'ds-bundle/README.md';
const INTERNAL_VAR = /^--(?:tw-[\w-]+|animate-[\w-]+|default-transition-[\w-]+)$/;
const CSS_DECLARATION =
  /(?<![\w-])(--[A-Za-z][\w-]*)(\s*:\s*)((?:[^;"'{}]|\([^)]*\)|"[^"]*"|'[^']*')*);(\s*\/\*\s*@kind\s+other\s*\*\/)?/g;
const CSS_PROPERTY_REGISTRATION =
  /(@property\s+)(--[A-Za-z][\w-]*)(\s*\{)(\s*\/\*\s*@kind\s+other\s*\*\/)?/g;
const NESTED_FOCUS_UTILITY_BLOCK =
  /^([ \t]*)(\.[^\n{]+)\s*\{\r?\n([ \t]*)&:(focus-within|focus-visible|focus)\s*\{\r?\n([\s\S]*?)\r?\n\3\}\r?\n\1\}/gm;
const NESTED_FOCUS_INTERNAL_DECLARATION = /--tw-(?:ring-shadow|ring-color|outline-style)\s*:/;

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function deindentNestedBody(body, nestedIndent) {
  return body.replace(new RegExp(`^${escapeRegExp(`${nestedIndent}  `)}`, 'gm'), nestedIndent);
}

function flattenNestedFocusUtilities(css) {
  let flattened = 0;
  const nextCss = css.replace(
    NESTED_FOCUS_UTILITY_BLOCK,
    (match, selectorIndent, selector, nestedIndent, pseudo, body) => {
      if (!NESTED_FOCUS_INTERNAL_DECLARATION.test(body)) return match;

      flattened += 1;
      return `${selectorIndent}${selector.trimEnd()}:${pseudo} {\n${deindentNestedBody(body, nestedIndent)}\n${selectorIndent}}`;
    },
  );

  return { css: nextCss, flattened };
}

function markDeclarations(css) {
  let marked = 0;
  const nextCss = css.replace(
    CSS_DECLARATION,
    (match, name, separator, value, existingKind) => {
      if (!INTERNAL_VAR.test(name) || existingKind) return match;

      marked += 1;
      return `${name}${separator}${value};${KIND_OTHER}`;
    },
  );

  return { css: nextCss, marked };
}

function markPropertyRegistrations(css) {
  let marked = 0;
  const nextCss = css.replace(
    CSS_PROPERTY_REGISTRATION,
    (match, prefix, name, openingBrace, existingKind) => {
      if (!INTERNAL_VAR.test(name) || existingKind) return match;

      marked += 1;
      return `${prefix}${name}${openingBrace}${KIND_OTHER}`;
    },
  );

  return { css: nextCss, marked };
}

function tokenFamily(name) {
  if (/color|bg-|fg-|text-|fill|border-(?!radius|width)|surface/i.test(name)) return 'color';
  if (/space|gap|pad|margin|inset|-p-|-m-/i.test(name)) return 'spacing';
  if (/font|line-height|letter|weight|tracking/i.test(name)) return 'typography';
  if (/radius|rounded/i.test(name)) return 'radius';
  if (/shadow|elevation/i.test(name)) return 'shadow';
  return 'other';
}

function tokenOverview(tokenNames) {
  const families = { color: [], spacing: [], typography: [], radius: [], shadow: [], other: [] };
  for (const name of [...tokenNames].sort()) families[tokenFamily(name)].push(name);

  return Object.entries(families)
    .filter(([, names]) => names.length)
    .map(([family, names]) => {
      const examples = names.slice(0, 3).map((name) => `\`${name}\``).join(', ');
      return `- **${family}** (${names.length}): ${examples}${names.length > 3 ? ', ...' : ''}`;
    })
    .join('\n');
}

function collectTokenNames(cssByFile) {
  const tokenNames = new Set();
  for (const css of cssByFile.values()) {
    for (const match of css.matchAll(CSS_DECLARATION)) {
      const name = match[1];
      if (!INTERNAL_VAR.test(name)) tokenNames.add(name);
    }
  }

  return tokenNames;
}

function updateReadme(cssByFile) {
  if (!existsSync(README_FILE)) return false;

  const readme = readFileSync(README_FILE, 'utf8');
  const tokenSection = /## Tokens\n\n[\s\S]*?\n\n## Components/;
  if (!tokenSection.test(readme)) return false;

  const tokenNames = collectTokenNames(cssByFile);
  const source = readme.match(/\d+ CSS custom properties from ([^.]+)\./)?.[1] ?? 'the design system';
  const overview = tokenOverview(tokenNames);
  const replacement =
    `## Tokens\n\n${tokenNames.size} CSS custom properties from ${source}. Names are\n` +
    'preserved verbatim from upstream. Tailwind runtime internals are marked\n' +
    '`@kind other` in the compiled CSS and omitted here.\n\n' +
    `${overview}\n\n## Components`;

  writeFileSync(README_FILE, readme.replace(tokenSection, replacement));
  return true;
}

let totalDeclarations = 0;
let totalRegistrations = 0;
let totalFlattened = 0;
const cssByFile = new Map();
for (const file of CSS_FILES) {
  if (!existsSync(file)) continue;

  const original = readFileSync(file, 'utf8');
  const flattened = flattenNestedFocusUtilities(original);
  const declarations = markDeclarations(flattened.css);
  const registrations = markPropertyRegistrations(declarations.css);

  if (registrations.css !== original) writeFileSync(file, registrations.css);
  cssByFile.set(file, registrations.css);

  totalFlattened += flattened.flattened;
  totalDeclarations += declarations.marked;
  totalRegistrations += registrations.marked;
}

const readmeUpdated = updateReadme(cssByFile);

console.error(
  `  strip-tw-vars: flattened ${totalFlattened} nested focus utility block(s),` +
    ` marked ${totalDeclarations} Tailwind internal declaration(s)` +
    ` and ${totalRegistrations} @property registration(s) as other` +
    `${readmeUpdated ? '; updated README token summary' : ''}`,
);
