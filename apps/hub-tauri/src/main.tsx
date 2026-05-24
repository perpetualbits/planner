/* main.tsx — SolidJS application entry point.
 *
 * Mounts the App component into the #root div defined in index.html.
 */

import { render } from "solid-js/web";
import App from "./App";

const root = document.getElementById("root");
if (root) {
  render(() => <App />, root);
}
