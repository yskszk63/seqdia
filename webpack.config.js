const CopyWebpackPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const path = require("path");

const docs = path.resolve(__dirname, "docs");

module.exports = {
  mode: "production",
  experiments: {
    asyncWebAssembly: true,
  },
  entry: {
      index: ["./index.js", "./app.css"],
  },
  output: {
    path: docs,
    filename: "[name].js",
  },
  devServer: {
    static: {
      directory: docs,
    },
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
    new CopyWebpackPlugin({
      patterns: [
        { from: 'index.html' },
        { from: 'logo.png' },
      ]
    }),
    new WasmPackPlugin({
      crateDirectory: __dirname,
    }),
  ],
};
