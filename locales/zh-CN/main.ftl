download_command=从菠萝包/刺猬猫上下载小说
search_command=从菠萝包/刺猬猫上搜索小说
info_command=打印菠萝包/刺猬猫小说信息
favorites_command=显示收藏夹小说信息
transform_command=转换 pandoc 风格的 markdown 文件
check_command=检查 markdown 文件格式
build_command=从 markdown 文件构建小说
zip_command=压缩 epub 文件夹
unzip_command=解压缩 epub 文件
real_cugan_command=运行 realcugan-ncnn-vulkan
update_command=升级自身
completions_command=生成 shell 补全到标准输出

novel_id=小说的 novel id
format=小说的输出格式
build=构建小说 [default: false]
delete=删除运行子命令所需的文件 [default: false]
open=使用网络浏览器或者 epub 阅读器打开构建后的小说 [default: false]
converts=进行转换 (繁简转换和自定义转换)
ignore_keyring=忽略 keyring 中保存的密码 [default: false]
maximum_concurrency=最大并发数
proxy=使用代理连接网络 [default: http://127.0.0.1:8080]
no_proxy=不使用代理连接网络 (忽略环境变量) [default: false]
cert=添加自定义证书 [default: {$cert_path}]
keywords=搜索小说使用的关键词
limit=搜索结果的最大数量
min_word_count=搜索条件：最小字数
tags=搜索条件：需要包含的标签
build_path=要构建的小说的路径
markdown_path=要转换的 markdown 文件的路径
epub_dir_path=要压缩的 epub 文件夹的路径
level=Zip 压缩等级
epub_path=要解压缩的 epub 文件路径
shell=要生成补全的 shell
verbose=使用详细的输出
quiet=不打印日志 [default: false]
source=指定小说的来源
image_path=要运行 realcugan-ncnn-vulkan 的图片所在路径，如果不指定，则为当前目录

login_msg={$emoji} 登录成功，昵称：{$name}
start_msg={$emoji} 开始下载小说：{$name}
download_complete_msg={$emoji} 小说下载完成
build_complete_msg={$emoji} 小说构建完成
build_msg={$emoji} 开始构建 {$type} 格式输出
