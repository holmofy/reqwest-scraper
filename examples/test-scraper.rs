use scraper::{Html, Selector};
use skyscraper::{
    html,
    xpath::{self, XpathItemTree},
};

fn main() {
    let html = r#"
    <div id="s-top-left" class="s-top-left-new s-isindex-wrap">
        <a href="http://news.baidu.com" target="_blank" class="mnav c-font-normal c-color-t">新闻</a>
        <a href="https://www.hao123.com?src=from_pc_logon" target="_blank" class="mnav c-font-normal c-color-t">hao123</a>
        <a href="http://map.baidu.com" target="_blank" class="mnav c-font-normal c-color-t">地图</a>
        <a href="http://tieba.baidu.com/" target="_blank" class="mnav c-font-normal c-color-t">贴吧</a>
        <a href="https://haokan.baidu.com/?sfrom=baidu-top" target="_blank" class="mnav c-font-normal c-color-t">视频</a>
        <a href="http://image.baidu.com/" target="_blank" class="mnav c-font-normal c-color-t">图片</a>
        <a href="https://pan.baidu.com?from=1026962h" target="_blank" class="mnav c-font-normal c-color-t">网盘</a>
        <a href="https://wenku.baidu.com/?fr=bdpcindex" target="_blank" class="mnav c-font-normal c-color-t">文库</a>
    </div>
    "#;

    let fragment = Html::parse_fragment(html);
    let selector = Selector::parse("#s-top-left > a").unwrap();

    for element in fragment.select(&selector) {
        println!("{}", element.text().collect::<Vec<_>>().join(""))
    }

    let doc = html::parse(html).unwrap();
    let tree = XpathItemTree::from(&doc);

    let xpath = xpath::parse("//div").unwrap();

    let set = xpath.apply(&tree).unwrap();

    println!("{}", set.len());

    let div = set.iter().next().unwrap();
    let node = div.as_node().unwrap().as_tree_node().unwrap();

    println!("{}", node.data.as_element_node().unwrap().get_attribute("id").unwrap());
}
