const CopyWebpackPlugin = require("copy-webpack-plugin");
const HtmlWebpackPlugin = require('html-webpack-plugin');
const FaviconsWebpackPlugin = require('favicons-webpack-plugin');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const path = require("path");

const docs = path.resolve(__dirname, "docs");

module.exports = {
  mode: "production",
  entry: {
      index: "./bootstrap.js",
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
          "style-loader",
          {
            loader: "css-loader",
            options: {
              url: false
            }
          }
        ]
      }
    ]
  },
  plugins: [
    new CopyWebpackPlugin(['index.html']),
    new FaviconsWebpackPlugin({
      prefix: 'assets/',
      publicPath: '',
    }),
    new WasmPackPlugin({
      crateDirectory: __dirname,
    }),
  ],
};
