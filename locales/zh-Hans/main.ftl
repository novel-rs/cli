sign_command = 签到并显示当前金钱数量
download_command = 下载小说
search_command = 搜索小说
info_command = 显示小说信息
read_command = 阅读小说
bookshelf_command = 显示书架中的小说
template_command = 生成 pandoc 风格的 markdown 文件模板
transform_command = 转换 pandoc 风格的 markdown 文件（扩展名为 txt 或者 md）
check_command = 检查 pandoc 风格的 markdown 文件的格式和内容（扩展名为 txt 或者 md）
build_command = 从 pandoc 风格的 markdown 文件（扩展名为 txt 或者 md）或 mdBook 文件夹构建小说
zip_command = 压缩 EPUB 文件夹
unzip_command = 解压缩 EPUB 文件
real_cugan_command = 运行 realcugan-ncnn-vulkan 来超分辨率图片
update_command = 检测更新，从 GitHub 上下载文件，并替换
completions_command = 生成 shell 补全到标准输出

novel_id = 小说的编号
format = 小说的输出格式
delete = 删除运行子命令所需的文件 [default: false]
open = 使用网络浏览器或者 EPUB 阅读器打开构建后的小说 [default: false]
converts = 将文本内容进行转换
ignore_keyring = 忽略 Keyring 中保存的密码 [default: false]
maximum_concurrency = 最大并发数
proxy = 使用代理连接网络 [default: http://127.0.0.1:8080]
no_proxy = 不使用代理连接网络 (忽略环境变量) [default: false]
cert = 添加自定义证书 [default: {$cert_path}]
keyword = 搜索小说使用的关键词
limit = 搜索结果的最大数量
show_categories = 显示所有类别名称
show_tags = 显示所有标签名称
min_word_count = 搜索条件：最小字数
max_word_count = 搜索条件：最大字数
update_days = 搜索条件：最后更新到今天的天数
is_finished = 搜索条件：是否已完结
is_vip = 搜索条件：是否是 VIP 小说
category = 搜索条件：类别名称
tags = 搜索条件：包含的标签
excluded_tags = 搜索条件：不包含的标签
build_path = 构建小说所使用的 pandoc 风格的 markdown 文件的路径（扩展名为 txt 或者 md），或包含该文件的文件夹的路径，或 mdBook 文件夹的路径
file_path = pandoc 风格的 markdown 文件的路径（扩展名为 txt 或者 md），或包含该文件的文件夹的路径
epub_dir_path = 要压缩的 EPUB 文件夹的路径
epub_path = 要解压缩的 EPUB 文件路径
shell = 要生成补全的 shell
verbose = 使用详细的输出
quiet = 不打印日志 [default: false]
backtrace = 打印 backtrace 信息
source = 小说的来源
image_path = 要运行 realcugan-ncnn-vulkan 的图片所在路径，如果不指定，则为当前目录
skip_login = 跳过下载小说时的登录
novel_name = 小说的名字
cover_image = 小说的封面图片

login_msg = {$emoji} 登录成功，昵称：{$arg}
start_msg = {$emoji} 开始下载小说：{$arg}
download_complete_msg = {$emoji} 小说下载完成
build_complete_msg = {$emoji} 小说构建完成
build_msg = {$emoji} 开始构建 {$arg} 格式输出

download_failed_msg = 无法下载该章节
inaccessible_msg = 该章节未购买或者该章节无法访问

enter_user_name = 请输入用户名
enter_password = 请输入密码

sign_in_successfully = 签到成功
current_money = 当前金钱：
