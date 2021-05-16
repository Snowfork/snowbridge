const Blake2b = artifacts.require("Blake2b");
const BigNumber = require("bignumber.js");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

contract("Blake2b", function () {
  let blake2b;
  beforeEach(async function () {
    blake2b = await Blake2b.new();
  });

  it("should hash correctly", async function () {
    const cases = [
      {
        in: "0x",
        out: "786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce",
      },
      {
        in: "0x00",
        out: "2fa3f686df876995167e7c2e5d74c4c7b6e48f8068fe0e44208344d480f7904c36963e44115fe3eb2a3ac8694c28bcb4f5a0f3276f2e79487d8219057a506e4b",
      },
      {
        in: "0x0001",
        out: "1c08798dc641aba9dee435e22519a4729a09b2bfe0ff00ef2dcd8ed6f8a07d15eaf4aee52bbf18ab5608a6190f70b90486c8a7d4873710b1115d3debbb4327b5",
      },
      {
        in: "0x000102",
        out: "40a374727302d9a4769c17b5f409ff32f58aa24ff122d7603e4fda1509e919d4107a52c57570a6d94e50967aea573b11f86f473f537565c66f7039830a85d186",
      },
      {
        in: "0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f",
        out: "2319e3789c47e2daa5fe807f61bec2a1a6537fa03f19ff32e87eecbfd64b7e0e8ccff439ac333b04f19b0c4ddd11a61e24ac1fe0f10a039806c5dcc0da3d115",
      },
    ];

    for (const c of cases) {
      expect(
        (await blake2b.blake2b(c.in, "0x", 64)).reduce((s, e) => {
          return s + e.toString(16);
        }, "")
      ).to.equal(c.out);
    }
  });
});
