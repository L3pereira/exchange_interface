const path = require('path');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const fs = require('fs');

function get_config(match, p1, offset, string) {
  let file_path = path.resolve(__dirname, '../config.json');
  let  rawdata = fs.readFileSync(file_path);
  let data = JSON.parse(rawdata);
  return data['client_websocket'];
}

module.exports = {
  mode: 'development',
  entry: [
    './www/src/index.js'
  ],
  module: {
    rules: [
      {
        test: /\.(js|jsx)$/,
        exclude: /node_modules/,
        
        use: {
          loader: "babel-loader",

        }
      },
      {
        test: /\.html$/i,
        loader: "html-loader",
      },
      {
        test: /\.css$/i,
        use: [
          // 'style-loader', 
          MiniCssExtractPlugin.loader,
          'css-loader'],
        // use: ['style-loader', 'css-loader'],
      },
      {
        test: /\.(png|svg|jpg|jpeg|gif)$/i,
        type: 'asset/resource',
      },
      {
        test: /\.(woff|woff2|eot|ttf|otf)$/i,
        type: 'asset/resource',
      },
      {
        test: /\.js$/,
        loader: 'string-replace-loader',
        options: {
          multiple: [
            { search: '@@@IP@@@', replace: get_config }
         ]
        }
      }
    ],
  },
  devtool: 'inline-source-map',
  plugins: [
    new MiniCssExtractPlugin({
      filename: '[name].css',
      chunkFilename: '[id].css'
    }),
    new HtmlWebpackPlugin({
      template: 'www/src/template.html',
      // filename: 'index.html',
      inject: 'body',
      hash:false,
    }),

    new CleanWebpackPlugin(),


  ],
  output: {
    filename: '[name].bundle.js',
    path: path.resolve(__dirname, 'www/dist'),
    clean: true,
  },
};