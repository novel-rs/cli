sign_command = 簽到並顯示當前金錢數量
download_command = 下載小說
search_command = 搜索小說
info_command = 顯示小說信息
read_command = 閱讀小說
bookshelf_command = 顯示書架中的小說
template_command = 生成 pandoc 風格的 markdown 文件模板
transform_command = 轉換 pandoc 風格的 markdown 文件（擴展名爲 txt 或者 md）
check_command = 檢查 pandoc 風格的 markdown 文件的格式和內容（擴展名爲 txt 或者 md）
build_command = 從 pandoc 風格的 markdown 文件（擴展名爲 txt 或者 md）或 mdBook 文件夾構建小說
zip_command = 壓縮 EPUB 文件夾
unzip_command = 解壓縮 EPUB 文件
real_cugan_command = 運行 realcugan-ncnn-vulkan 來超分辨率圖片
update_command = 檢測更新，從 GitHub 上下載文件，並替換
completions_command = 生成 shell 補全到標準輸出

novel_id = 小說的編號
format = 小說的輸出格式
delete = 刪除運行子命令所需的文件 [default: false]
open = 使用網絡瀏覽器或者 EPUB 閱讀器打開構建後的小說 [default: false]
converts = 將文本內容進行轉換
ignore_keyring = 忽略 Keyring 中保存的密碼 [default: false]
maximum_concurrency = 最大併發數
proxy = 使用代理連接網絡 [default: http://127.0.0.1:8080]
no_proxy = 不使用代理連接網絡 (忽略環境變量) [default: false]
cert = 添加自定義證書 [default: {$cert_path}]
keyword = 搜索小說使用的關鍵詞
limit = 搜索結果的最大數量
show_categories = 顯示所有類別名稱
show_tags = 顯示所有標籤名稱
min_word_count = 搜索條件：最小字數
max_word_count = 搜索條件：最大字數
update_days = 搜索條件：最後更新到今天的天數
is_finished = 搜索條件：是否已完結
is_vip = 搜索條件：是否是 VIP 小說
category = 搜索條件：類別名稱
tags = 搜索條件：包含的標籤
excluded_tags = 搜索條件：不包含的標籤
build_path = 構建小說所使用的 pandoc 風格的 markdown 文件的路徑（擴展名爲 txt 或者 md），或包含該文件的文件夾的路徑，或 mdBook 文件夾的路徑
file_path = pandoc 風格的 markdown 文件的路徑（擴展名爲 txt 或者 md），或包含該文件的文件夾的路徑
epub_dir_path = 要壓縮的 EPUB 文件夾的路徑
epub_path = 要解壓縮的 EPUB 文件路徑
shell = 要生成補全的 shell
verbose = 使用詳細的輸出
quiet = 不打印日誌 [default: false]
backtrace = 打印 backtrace 信息
source = 小說的來源
image_path = 要運行 realcugan-ncnn-vulkan 的圖片所在路徑，如果不指定，則爲當前目錄
skip_login = 跳過下載小說時的登錄
novel_name = 小說的名字
cover_image = 小說的封面圖片
basic_check = 只進行基本檢查

login_msg = {$emoji} 登錄成功，暱稱：{$arg}
start_msg = {$emoji} 開始下載小說：{$arg}
download_complete_msg = {$emoji} 小說下載完成
build_complete_msg = {$emoji} 小說構建完成
build_msg = {$emoji} 開始構建 {$arg} 格式輸出

download_failed_msg = 無法下載該章節
inaccessible_msg = 該章節未購買或者該章節無法訪問

enter_user_name = 請輸入用戶名
enter_password = 請輸入密碼

sign_in_successfully = 簽到成功
current_money = 當前金錢：
