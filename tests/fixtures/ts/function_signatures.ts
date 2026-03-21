export function add(a: number, b: number): number {
  return a + b;
}

export function greet(name: string, greeting?: string): string {
  return `${greeting ?? "Hello"}, ${name}!`;
}
