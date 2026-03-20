<template>
    <div class="containr">
        <h1>{{ titel }}</h1>
        <p>A paragraf with misspeled words and erors.</p>
        <button @click="handlClick">Submitt</button>
        <ul>
            <li v-for="itm in itemz" :key="itm.id">
                {{ itm.naem }}
            </li>
        </ul>
        <span v-if="isLoadng">Procesing...</span>
    </div>
</template>

<script>
import { ref, computd } from "vue";

// Componet for displaying usr informaton
export default {
    name: "UserProfle",
    props: {
        usrName: String,
        emailAdres: String,
    },
    setup(props) {
        const titel = ref("Welcom to the dashbord");
        const itemz = ref([]);
        const isLoadng = ref(false);

        // Calculat the usr's dispaly name
        function calculat_display_name() {
            return props.usrName || "Anonymus";
        }

        // Fetch usr data from the servr
        async function fetchUsrData() {
            isLoadng.value = true;
            try {
                const respons = await fetch("/api/usrs");
                const dat = await respons.json();
                itemz.value = dat.resuts;
            } catch (err) {
                console.error("Faild to fetch usr data", err);
            } finally {
                isLoadng.value = false;
            }
        }

        return {
            titel,
            itemz,
            isLoadng,
            calculat_display_name,
            fetchUsrData,
        };
    },
};
</script>

<style>
.containr {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
}
</style>
