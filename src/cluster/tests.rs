use super::*;
use unicode_segmentation::UnicodeSegmentation;

test_files!("./../../../../");

#[test]
fn at_index() {
    const TAKE: usize = 10;
    let lines = FILES.iter().map(|file| file.lines().take(TAKE)).flatten();

    for str in lines {
        for (indice, cluster) in str.grapheme_indices(true) {
            let cluster = Cluster::from_raw(cluster, utils::width(cluster) as u8);

            for i in indice..indice + cluster.len() {
                let (ret_i, ret_cluster) = Cluster::at_index(str, i);

                assert!(ret_i == indice);
                assert!(ret_cluster == Some(cluster));
            }
        }

        let (i, cluster) = Cluster::at_index(str, str.len());
        assert!(i == str.len());
        assert!(cluster == None);
    }
}
