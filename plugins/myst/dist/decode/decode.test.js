import { decode } from ".";
test("empty myst", async () => {
    const [node, info] = decode("");
    expect(JSON.stringify(node, null, "  ")).toMatchSnapshot();
});
