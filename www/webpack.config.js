const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js"
  },
  experiments: {
    asyncWebAssembly: true,
  },
  mode: "development",
  plugins: [
      // new CopyWebpackPlugin(['index.html']),
      new CopyWebpackPlugin({
          patterns: [
              { from: "index.html", to: "." },
              { context: "public", from: "**/*", to: "." },
          ],
      }),
  ],

  devServer: {
    static: {
      directory: path.join(__dirname, 'public'),
    }
  },
};
