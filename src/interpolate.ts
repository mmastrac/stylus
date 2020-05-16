/* 
 * Warning! This interpolatation function does _not_ protect against remote attackers. Instead it just
 * tries to remove things from the environment that would normally be confusing.
 * 
 * This whole thing is super hokey and could probably be replaced by a safer worker.
 */
export function interpolateUnsafely(s: string, env: any): string {
  let js = "";
  // Extract all the vars from the environment and put them into the local environment
  for (var x in env) {
    js += ("var " + x + " = env." + x + ";");
  }
  return eval(js + "`" + s + "`");
}
