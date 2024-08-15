const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');

module.exports = {
  entry: './src/index.tsx',
  devtool: 'inline-source-map',
  module: {
    rules: [
      {
        // Allow 'import * from "./foo.tsx"'
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
      {
        // allow 'import "foo.css"' and '@import "foo.css" in css files
        test: /\.css$/i,
        use: ['style-loader', 'css-loader'],
        generator: { filename: 'styles/[name].[contenthash][ext]' },
      },
      {
        // allow 'import Icon from "./icon.png"'
        test: /\.(png|svg|jpg|jpeg|gif)$/i,
        type: 'asset/resource',
        generator: { filename: 'images/[name].[contenthash][ext]' },
      },
      {
        // allow CSS @url('./my-font.woff2')" style font loading
        test: /\.(woff|woff2|eot|ttf|otf)$/i,
        type: 'asset/resource',
        generator: { filename: 'fonts/[name].[contenthash][ext]' },
      },
    ],
  },
  plugins: [
    // Use our index.html as a starting point (to add script links etc to)
    // and make sure to use/copy over the favicon too.
    new HtmlWebpackPlugin({
      favicon: './assets/favicon.svg',
      template: './assets/index.html',
    }),
  ],
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
  },
  output: {
    filename: 'main.[contenthash].js',
    path: path.resolve(__dirname, 'build'),
    clean: true,
  },
};
