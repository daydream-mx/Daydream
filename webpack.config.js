const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const TerserPlugin = require('terser-webpack-plugin');
const WaitPlugin = require('./startup_helper/WaitPlugin');

const distPath = path.resolve(__dirname, "dist");
const appConfig = (env, argv) => {
    return {
        devServer: {
            contentBase: distPath,
            compress: argv.mode === 'production',
            port: 8888,
            historyApiFallback: true
        },
        entry: ['./bootstrap.js'],
        output: {
            path: distPath,
            filename: "daydream.js",
            webassemblyModuleFilename: "daydream.wasm"
        },
        optimization: {
            minimize: true,
            minimizer: [new TerserPlugin({
                parallel: true,
            })],
        },
        module: {
            rules: [
                {
                    test: /\.(png|jpe?g|gif|webp)$/i,
                    loader: 'file-loader',
                    options: {
                        name: '[name].[ext]',
                    },
                },
                {
                    test: /\.s[ac]ss$/i,
                    include: [path.resolve(__dirname, "static"), path.resolve(__dirname, "node_modules")],
                    use: [
                        // fallback to style-loader in development
                        argv.mode !== 'production'
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
        watch: argv.mode !== 'production',
        watchOptions: {
            poll: true
        },
        devtool: 'inline-source-map'
    };
};

// This config actually generates both
const workerConfig = {
    entry: "./startup_helper/worker/worker.js",
    target: "webworker",
    plugins: [
        new WaitPlugin('./dist/daydream.wasm', 100, 600000)
    ],
    resolve: {
        extensions: [".js", ".wasm"]
    },
    output: {
        path: distPath,
        filename: "worker.js"
    },
    devtool: 'inline-source-map'
};

module.exports = (env, argv) => {
    return [appConfig(env, argv), workerConfig]
};
