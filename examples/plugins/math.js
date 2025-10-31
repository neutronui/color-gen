// Example JS plugin providing math transforms:
// - scale: multiply by a factor
// - addPx: add N px to string or dimension

function scale(input, args, ctx) {
  const factor = Number(args?.[0] ?? 1);
  if (typeof input === 'number') return input * factor;
  if (typeof input === 'string') return `calc(${input} * ${factor})`;
  if (input && input.type === 'dimension') {
    return { type: 'dimension', value: input.value * factor, unit: input.unit };
  }
  return input;
}

function addPx(input, args, ctx) {
  const amount = Number(args?.[0] ?? 0);
  if (typeof input === 'string') return `calc(${input} + ${amount}px)`;
  if (input && input.type === 'dimension') {
    return { type: 'dimension', value: input.value + amount, unit: input.unit };
  }
  if (typeof input === 'number') return input + amount;
  return input;
}
