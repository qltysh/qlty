import * as hello from "src/unusedCode.js";
import { foo } from "src/mistake.js";

export default function() {
  hello();
  return 'Hello, world!';
}
