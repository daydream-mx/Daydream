const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');

const distPath = path.resolve(__dirname, "dist");
module.exports = (env, argv) => {
    return {
        devServer: {
            contentBase: distPath,
            compress: argv.mode === 'production',
            port: 8000,
            historyApiFallback: true
        },
        entry: './bootstrap.js',
        output: {
            path: distPath,
            filename: "daydream.js",
            webassemblyModuleFilename: "daydream.wasm"
        },
        module: {
            rules: [
                {
                    test: /\.s[ac]ss$/i,
                    include: [path.resolve(__dirname, "static"), path.resolve(__dirname, "node_modules")],
                    use: [
                        // fallback to style-loader in development
                        process.env.NODE_ENV !== 'production'
                            ? 'style-loader'
                            : MiniCssExtractPlugin.loader,
                        'css-loader',
                        'sass-loader',
                    ],
                },
            ],
        },
        plugins: [
            new CopyWebpackPlugin({
                patterns: [
                    /*{
                        from: './static', to: distPath
                    },*/
                    {
                        from: './node_modules/uikit/dist/js/uikit.min.js',
                        to: path.join(distPath, '/js/uikit.min.js')
                    },
                    {
                        from: './node_modules/uikit/dist/js/uikit-icons.min.js',
                        to: path.join(distPath, '/js/uikit-icons.min.js')
                    }
                ]
            }),
            new WasmPackPlugin({
                crateDirectory: ".",
                extraArgs: "--no-typescript",
            }),
            new MiniCssExtractPlugin({
                filename: '[name].css',
                chunkFilename: '[id].css',
            }),
            new HtmlWebpackPlugin({
                template: path.resolve(__dirname, "static/index.html")
            })
        ],
        watch: argv.mode !== 'production'
    };
};
