const CopyWebpackPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const path = require("path");

const docs = path.resolve(__dirname, "docs");

module.exports = {
  mode: "production",
  entry: {
      index: ["./index.js", "./app.css"],
  },
  output: {
    path: docs,
    filename: "[name].js",
  },
  devServer: {
    contentBase: docs,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          {
            loader: "file-loader",
            options: {
              name: "bundle.css",
            },
          },
          "extract-loader",
          "css-loader",
        ]
      }
    ]
  },
  plugins: [
    new CopyWebpackPlugin(['index.html']),
    new WasmPackPlugin({
      crateDirectory: __dirname,
    }),
  ],
};
