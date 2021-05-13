# 微信凭据服务器

## Usage
设置环境变量`WX_APPID`、`WX_SECRET`以及`SERVER_SECRET`并运行即可。

Endpoint: `yourdomain/token?key=<your_secret>`

## Changelog

**v0.1.1** 添加了`SERVER_SECRET`用于请求鉴权；将部分`static`移动到`AppState`中。

## Get
可以去Release中自行下载或者使用`cargo build --release`自行编译。
