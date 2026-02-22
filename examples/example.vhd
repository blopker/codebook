-- This is an exmple VHDL file with speling errors
-- for testing the codebook spell checker

library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

-- Entity declarashun for a simple calculater
entity calculatr is
    port (
        clk       : in  std_logic;
        resett    : in  std_logic;
        inputt    : in  std_logic_vector(7 downto 0);
        outpuut   : out std_logic_vector(7 downto 0)
    );
end entity calculatr;

-- Architectur definition
architecture behavorial of calculatr is
    signal intrnal_data : std_logic_vector(7 downto 0);
    constant max_valuue : integer := 255;
    variable tmp_resullt : integer := 0;
begin

    -- Main proccess block
    main_proccess : process(clk, resett)
    begin
        if resett = '1' then
            intrnal_data <= (others => '0');
        elsif rising_edge(clk) then
            intrnal_data <= inputt;
        end if;
    end process main_proccess;

    outpuut <= intrnal_data;

end architecture behavorial;
