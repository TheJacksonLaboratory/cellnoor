export async function load({ locals: { user } }) {
  console.log("root layout running");
  return { user };
}
