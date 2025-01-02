import { fileURLToPath } from "node:url";

import CopyWebpackPlugin from "copy-webpack-plugin";
import WasmPackPlugin from "@wasm-tool/wasm-pack-plugin";

const docs = fileURLToPath(import.meta.resolve("./docs"));

const config = {
  mode: "production",
  experiments: {
    asyncWebAssembly: true,
  },
  entry: {
      index: ["./index.js"],
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
          "style-loader",
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
      crateDirectory: import.meta.dirname,
    }),
  ],
};

export default config;
