// Example JS plugin with placeholder color transforms. These are simple string-based variants.
// In the future, you can integrate real color math or call into CSS color-mix.

function suffix(input, args, ctx) {
  const suf = String(args?.[0] ?? '');
  if (typeof input === 'string') return `${input}${suf}`;
  return input;
}

function wrapVar(input, args, ctx) {
  // wrap a token path into var(--path)
  const path = String(args?.[0] ?? '');
  const cssName = `--${path.replaceAll('.', '-')}`;
  return `var(${cssName})`;
}
