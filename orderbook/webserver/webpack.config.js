const path = require('path');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');


module.exports = {
  mode: 'development',
  entry: [
    './www/src/index.js'
  ],
  module: {
    rules: [
      {
        test: /\.html$/i,
        loader: "html-loader",
      },
    ],
  },
  devtool: 'inline-source-map',
  plugins: [
    // new HtmlWebpackPlugin({
    //     title: 'Output Management',
    // }),
    new CleanWebpackPlugin(),
    new HtmlWebpackPlugin({
      template: 'www/src/template.html',
      // filename: path.resolve(__dirname, 'www/src/index.html')
    }),
  ],
  output: {
    filename: '[name].bundle.js',
    path: path.resolve(__dirname, 'www/dist'),
    clean: true,
  },
};