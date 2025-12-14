//! ERC8040 ESG Token Interface
//! Interface for ESG-compliant tokens with environmental, social, and governance scores

use ethers::prelude::abigen;

abigen!(
    IERC8040,
    r#"[
        function mintWithESG(address to, uint256 amount, uint256 tokenId, string memory auditHash) external
        function getESGScore(uint256 tokenId) external view returns (uint8, uint8, uint8, uint256)
        function transferWithESG(address to, uint256 amount, uint256 tokenId) external returns (bool)
        function balanceOf(address account) external view returns (uint256)
        event MintWithESG(address indexed to, uint256 amount, uint256 indexed tokenId, string auditHash)
        event TransferWithESG(address indexed from, address indexed to, uint256 amount, uint256 indexed tokenId)
    ]"#
);
