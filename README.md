# `@ohos/json-bignumber`

ArkTs binding for converting a JavaScript value to and from a JSON string.

## Install

use`ohpm` to install package.

```shell
ohpm install @ohos/json-bignumber
```

## API

```ts
export interface Options {
  alwaysParseAsBig?: boolean
  useNativeBigInt?: boolean
  parseFloatAsBig?: boolean
}

export declare function parse(s: string, options?: Options | undefined | null): unknown

export declare function stringify(value: unknown): string

export declare class BigNumber {
  constructor(n: number | string | BigNumber, base?: number | undefined | null)
  static isBigNumber(value: unknown): boolean
  absoluteValue(): BigNumber
  abs(): BigNumber
  comparedTo(n: BigNumber): number
  decimalPlaces(): number
  dp(): number
  dividedBy(n: BigNumber): BigNumber
  div(n: BigNumber): BigNumber
  dividedToIntegerBy(n: BigNumber): BigNumber
  idiv(n: BigNumber): BigNumber
  negated(): BigNumber
  plus(n: BigNumber): BigNumber
  minus(n: BigNumber): BigNumber
  times(n: BigNumber): BigNumber
  exp(): BigNumber
  integerValue(): BigNumber
  modulo(n: BigNumber): BigNumber
  square(): BigNumber
  cube(): BigNumber
  sqrt(): BigNumber | null
  cbrt(): BigNumber
  toFixed(n?: number | undefined | null): string
  toExponential(n: number): string
  toString(): string
  toJSON(): unknown
}
```

## Usage

```ts
import {parse, stringify, BigNumber} from '@ohos/json-bignumber'

// parses a JSON string, constructing the JavaScript value or object described by the string
let json_str = '{"small":123,"big":1234567890123456789012345678901234567890,"float":1234567890.123e4567890}'
let obj = parse(json_str)

// converts a JavaScript value to a JSON string
let obj = {
    "small": 123,
    "big": 1234567890123456789012345678901234567890n,
    "float": new BigNumber("1234567890.123e4567890")
}
let json_str = stringify(obj)
```

## License

This project is licensed under the MIT License.