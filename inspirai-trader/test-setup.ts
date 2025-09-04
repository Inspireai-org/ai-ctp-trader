/**
 * Bun 测试环境设置
 */

// 全局测试工具类型定义
declare global {
  function describe(name: string, fn: () => void): void;
  function it(name: string, fn: () => void | Promise<void>): void;
  function beforeEach(fn: () => void | Promise<void>): void;
  function afterEach(fn: () => void | Promise<void>): void;
  function beforeAll(fn: () => void | Promise<void>): void;
  function afterAll(fn: () => void | Promise<void>): void;
  
  interface ExpectMatchers {
    toBe(expected: any): void;
    toEqual(expected: any): void;
    toBeGreaterThan(expected: number): void;
    toBeGreaterThanOrEqual(expected: number): void;
    toBeLessThan(expected: number): void;
    toBeLessThanOrEqual(expected: number): void;
    toContain(expected: any): void;
    toBeTruthy(): void;
    toBeFalsy(): void;
    toBeNull(): void;
    toBeUndefined(): void;
    toBeDefined(): void;
    toThrow(expected?: string | RegExp): void;
  }
  
  function expect(actual: any): ExpectMatchers;
}

// 模拟浏览器环境
if (typeof window === 'undefined') {
  global.window = {} as any;
  global.document = {} as any;
  global.localStorage = {
    getItem: () => null,
    setItem: () => {},
    removeItem: () => {},
    clear: () => {},
  } as any;
}

export {};